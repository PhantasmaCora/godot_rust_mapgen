use std::collections::HashSet;
use std::cmp::{Ord, Ordering};

use godot::prelude::*;
use godot::classes::FastNoiseLite;

use ndarray::Array;

use crate::datagrid::{DataGrid, GridElement, ElemType, Selection};
use crate::algorithm::{AlgorithmHelper, RectPrism};
use crate::algorithm::pathcarver::SearchMap;
use crate::algorithm::cellular_automata::CellAutoRule;


#[derive(GodotConvert, Var, Export, Default)]
#[godot(via = i64)]
pub enum CommandMode {
    #[default]
    Initialize,
    Expressions,
    SampleNoise,
    SampleNeighborhood,
    OuterWalls,
    DropFields,
    SetOps,
    RandomRooms,
    GetRoomCenters,
    SortList,
    ListToSel,
    SelToList,
    CarvePaths,
    ListInput,
    CellularAutomata,
    IntervalSelect,
    SelectFall,
}

#[derive(GodotConvert, Var, Export, Default, PartialEq, Eq, Clone, Copy)]
#[godot(via = i64)]
pub enum EdgeMode {
    #[default]
    Ignore,
    Loop,
    Clamp
}

#[derive(GodotConvert, Var, Export, Default, PartialEq, Eq, Clone, Copy)]
#[godot(via = i64)]
pub enum SortAxis {
    X,
    #[default]
    Y,
    Z
}

#[derive(GodotConvert, Var, Export, Default, PartialEq, Eq, Clone, Copy)]
#[godot(via = i64)]
pub enum SetBoolean {
    #[default]
    Union,
    Intersection,
    Difference,
}

#[derive(GodotClass, Default)]
#[class(tool, init, base=Resource)]
pub struct MapGenExpression {
    #[export]
    pub name: StringName,
    #[export]
    pub expr: GString,
    #[export]
    pub result_kind: ElemType,
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct Neighborhood {
    #[export]
    pub offsets: godot::prelude::Array<Vector3i>,
    #[export]
    pub summing_expr: GString,
    #[export]
    pub accumulator_base: f64,
}


#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct MapGenCommand {
    base: Base<Resource>,
    #[export]
    pub mode: CommandMode,

    #[export]
    pub seed_salt: i64,

    #[export]
    pub source: GString,

    #[export]
    pub save: GString,

    #[export_group(name = "Initialize mode")]
    #[export]
    pub init_size: Vector3i,

    #[export_group(name = "Expressions mode")]
    #[export]
    pub expression_list: godot::prelude::Array<Gd<MapGenExpression>>,

    #[export_group(name = "SampleNoise mode")]
    #[export]
    pub noise: Option<Gd<FastNoiseLite>>,

    #[export_group(name = "SampleNeighborhood mode")]
    #[export]
    pub neighborhood: Option<Gd<Neighborhood>>,
    #[export]
    pub edge_mode: EdgeMode,

    #[export_group(name = "DropFields mode")]
    #[export]
    pub to_drop: godot::prelude::Array<GString>,

    #[export_group(name = "SetOps mode")]
    #[export]
    pub second_source: GString,
    #[export]
    pub operation: SetBoolean,

    #[export_group(name = "RandomRooms mode")]
    #[export]
    pub allow_overlap: bool,
    #[export]
    pub count: i64,
    #[export]
    pub min_size : Vector3i,
    #[export]
    pub max_size : Vector3i,
    #[export]
    pub min_within : Vector3i,
    #[export]
    pub max_within : Vector3i,
    #[export]
    pub save_union : GString,

    #[export_group(name = "SortList mode")]
    #[export]
    pub sort_axis: SortAxis,
    #[export]
    pub reverse: bool,

    #[export_group(name = "CarvePaths mode")]
    #[export]
    pub max_slope: f32,
    #[export]
    pub vertical_skew: f64,
    #[export]
    pub points_list: GString,

    #[export_group(name = "ListInput mode")]
    #[export]
    pub position_list: godot::prelude::Array<Vector3i>,

    #[export_group(name = "CellularAutomata mode")]
    #[export]
    pub ca_rule: Option<Gd<CellAutoRule>>,
    #[export]
    pub steps: i64,
    #[export]
    pub apply_min: Vector3i,
    #[export]
    pub apply_max: Vector3i,

    #[export_group(name = "IntervalSelect mode")]
    #[export]
    pub interval: Vector3i,
    #[export]
    pub offset: Vector3i,

    #[export_group(name = "SelectFall mode")]
    #[export]
    pub solid: GString,
    #[export]
    pub sf_reverse: bool,
    #[export]
    pub column: bool,
}


#[derive(GodotConvert, Var, Export, PartialEq, Eq)]
#[godot(via = i64)]
pub enum NeedsInput {
    No,
    One
}


#[godot_api]
impl MapGenCommand {
    #[func]
    pub fn needs_input(&self) -> NeedsInput {
        match self.mode {
            CommandMode::Initialize => NeedsInput::No,
            _ => NeedsInput::One,
        }
    }
}

impl MapGenCommand {

    pub fn run_none( &self, _seed: i64, name: String ) -> Result<DataGrid, String> {
        match self.mode {
            CommandMode::Initialize => {
                if self.init_size.x < 1 || self.init_size.y < 1 || self.init_size.z < 1 {
                    return Err( format!("Initialize command '{}' was set to invalid size!", name ) );
                }
                return Ok( DataGrid::sized( ( self.init_size.x as usize, self.init_size.y as usize, self.init_size.z as usize ) ) );
            },
            _ => { return Err( format!("Attempted to run command '{}' by without input, but input was expected!", name ) ); },
        }
    }

    pub fn run_one( &self, seed: i64, mut input: DataGrid, name: String ) -> Result<DataGrid, String> {
        match self.mode {
            CommandMode::Expressions => {
                for e in self.expression_list.iter_shared() {
                    let e = e.bind();

                    let res = input.parallel_expr( e.name.to_string(), &e.expr, &e.result_kind );
                    if res.is_err() {
                        return Err( format!("Error running command '{}': Expression '{}' execution failed with '{:?}'", name, e.name, res.unwrap_err() ) );
                    }
                }
                return Ok(input);

            },
            CommandMode::SampleNoise => {
                if self.noise.is_none() {
                    return Err( format!("SampleNoise command '{}' had no noise supplied!", name ) );
                }
                let mut noise = self.noise.clone().unwrap();
                noise.set_seed( (seed + self.seed_salt) as i32 );
                let sample = Array::from_shape_fn(input.size, | (x, y, z) | {noise.get_noise_3d( x as f32, y as f32, z as f32 ) as f64} );
                input.elements.insert( self.save.to_string(), GridElement::Float(sample) );
                return Ok(input);

            },
            CommandMode::SampleNeighborhood => {
                if self.neighborhood.is_none() {
                    return Err( format!("SampleNeighborhood command '{}' had no neighborhood supplied!", name ) );
                }
                let res = input.sample_neighborhood( self.neighborhood.as_ref().unwrap(), self.edge_mode, &self.source.to_string(), &self.save.to_string() );
                if res.is_err() {
                    return Err( format!("Error running command '{}': Sampling execution failed with '{:?}'", name, res.unwrap_err() ) );
                }
                return Ok(input);

            },
            CommandMode::OuterWalls => {
                if self.save.is_empty() {
                    return Err( format!("OuterFaces command '{}' had empty save string supplied!", name ) );
                }
                let mut select = Box::new( HashSet::<(i64, i64, i64)>::new() );
                for x in 0..input.size.0 {
                    for y in 0..input.size.1 {
                        select.insert( (x as i64, y as i64, 0) );
                        select.insert( (x as i64, y as i64, input.size.2 as i64) );
                    }

                    for z in 0..input.size.2 {
                        select.insert( (x as i64, 0, z as i64) );
                        select.insert( (x as i64, input.size.1 as i64, z as i64) );
                    }
                }

                for y in 0..input.size.1 {
                    for z in 0..input.size.2 {
                        select.insert( (0, y as i64, z as i64) );
                        select.insert( (input.size.0 as i64, y as i64, z as i64) );
                    }
                }

                input.elements.insert( self.save.to_string(), GridElement::Sel(select) );
                return Ok(input);
            },
            CommandMode::DropFields => {
                for f in self.to_drop.iter_shared() {
                    let _ = input.elements.remove( &f.to_string() );
                }
                return Ok(input);
            },
            CommandMode::RandomRooms => {
                let sizes = RectPrism{ min:( self.min_size.x as usize, self.min_size.y as usize, self.min_size.z as usize ), max:( self.max_size.x as usize, self.max_size.y as usize, self.max_size.z as usize ) };
                let area = RectPrism{ min:( self.min_within.x as usize, self.min_within.y as usize, self.min_within.z as usize ), max:( self.max_within.x as usize, self.max_within.y as usize, self.max_within.z as usize ) };

                let res = AlgorithmHelper::random_rooms( self.count, seed + self.seed_salt, area, sizes, self.allow_overlap );

                if let Ok( ( vec, uni ) ) = res {
                    input.elements.insert( self.save.to_string(), GridElement::Rooms( vec ) );

                    if !self.save_union.is_empty() {
                        input.elements.insert( self.save_union.to_string(), GridElement::Sel( uni ) );
                    }

                    return Ok(input);
                } else {
                    return Err( format!("Error running command '{}': Random rooms failed with '{:?}'", name, res.err().unwrap() ) );
                }
            },
            CommandMode::SortList => {
                if let Some(GridElement::List(mut vec)) = input.elements.remove( &self.source.to_string() ) {
                    let slice = &mut vec[..];

                    let sorter: &dyn Fn( &(i64, i64, i64), &(i64, i64, i64) ) -> Ordering;

                    if self.reverse {
                        match self.sort_axis {
                            SortAxis::X => { sorter = &| a, b | { a.0.cmp( &b.0 ).reverse() } },
                            SortAxis::Y => { sorter = &| a, b | { a.1.cmp( &b.1 ).reverse() } },
                            SortAxis::Z => { sorter = &| a, b | { a.2.cmp( &b.2 ).reverse() } }
                        }
                    } else {
                        match self.sort_axis {
                            SortAxis::X => { sorter = &| a, b | { a.0.cmp( &b.0 ) } },
                            SortAxis::Y => { sorter = &| a, b | { a.1.cmp( &b.1 ) } },
                            SortAxis::Z => { sorter = &| a, b | { a.2.cmp( &b.2 ) } }
                        }
                    }

                    slice.sort_by( sorter );

                    input.elements.insert( self.save.to_string(), GridElement::List( vec ) );
                    return Ok(input);
                } else {
                    return Err( format!("Attempted to run SortList command '{}' on a field that wasn't a List field!", name ) );
                }
            },
            CommandMode::CarvePaths => {
                if let Some(GridElement::Float(arr)) = input.elements.remove( &self.source.to_string() ) {
                    if let Some(GridElement::List(vec)) = input.elements.remove( &self.points_list.to_string() ) {
                        let sm = SearchMap{ weight_array: arr, max_slope: self.max_slope.abs(), vertical_skew: (self.vertical_skew as f32).abs() };
                        let mut uni = Box::new( HashSet::<(i64, i64, i64)>::new() );

                        for ridx in 0..(vec.len() - 1) {
                            let ca = vec[ridx];
                            let cb = vec[ridx + 1];
                            let result = sm.thstar( ca, cb );
                            if let Ok( path ) = result {
                                uni = Box::new( &*uni | &*path );
                            }
                        }

                        input.elements.insert( self.save.to_string(), GridElement::Sel(uni) );
                        return Ok(input);
                    } else {
                        return Err( format!("Attempted to run CarvePaths command '{}' without providing a set of rooms to connect!", name ) );
                    }
                } else {
                    return Err( format!("Attempted to run CarvePaths command '{}' without providing a (float) weights field!", name ) );
                }
            },
            CommandMode::SetOps => {
                if let Some(GridElement::Sel(a)) = input.elements.get( &self.source.to_string() ) {
                    if let Some(GridElement::Sel(b)) = input.elements.get( &self.second_source.to_string() ) {
                        let newset : Selection;
                        match self.operation {
                            SetBoolean::Union => {
                                newset = Box::new( &**a | &**b );
                            },
                            SetBoolean::Intersection => {
                                newset = Box::new( &**a & &**b );
                            },
                            SetBoolean::Difference => {
                                newset = Box::new( &**a - &**b );
                            },
                        }
                        input.elements.insert( self.save.to_string(), GridElement::Sel(newset) );
                        return Ok(input);
                    } else {
                        return Err( format!("Attempted to run SetOps command '{}' with a non-boolean second source!", name ) );
                    }
                } else {
                    return Err( format!("Attempted to run SetOps command '{}' with a non-boolean source!", name ) );
                }
            },
            CommandMode::GetRoomCenters => {
                if let Some(GridElement::Rooms(vec)) = input.elements.get( &self.source.to_string() ) {
                    let list = Vec::from_iter( vec.into_iter().map( |r| r.center ) );
                    input.elements.insert( self.save.to_string(), GridElement::List(list) );
                    return Ok(input);
                } else {
                    return Err( format!("Attempted to run GetRoomCenters command '{}' with a non-rooms source!", name ) );
                }
            },
            CommandMode::ListInput => {
                let mut ls = Vec::<(i64, i64, i64)>::new();
                for ivect in self.position_list.iter_shared() {
                    ls.push( ( ivect.x as i64, ivect.y as i64, ivect.z as i64 ) );
                }
                input.elements.insert( self.save.to_string(), GridElement::List(ls) );
                return Ok(input);
            },
            CommandMode::ListToSel => {
                if let Some(GridElement::List(vec)) = input.elements.remove( &self.source.to_string() ) {
                    let mut sel = Box::new( HashSet::<(i64, i64, i64)>::new() );
                    for pos in vec {
                        sel.insert(pos);
                    }
                    input.elements.insert( self.save.to_string(), GridElement::Sel(sel) );
                    return Ok(input);
                } else {
                    return Err( format!("Attempted to run ListToSel command '{}' with a non-list source!", name ) );
                }
            },
            CommandMode::SelToList => {
                if let Some(GridElement::Sel(sel)) = input.elements.remove( &self.source.to_string() ) {
                    let list = Vec::from_iter( sel.iter().map( |af| af.clone() ) );
                    input.elements.insert( self.save.to_string(), GridElement::List(list) );
                    return Ok(input);
                } else {
                    return Err( format!("Attempted to run SelToList command '{}' with a non-boolean source!", name ) );
                }
            },
            CommandMode::CellularAutomata => {
                if let Some(rule) = &self.ca_rule {
                    let data = input.elements.remove( &self.source.to_string() );
                    if let Some(ge) = data {
                        let res = rule.bind().run( ge, RectPrism{ min:(self.apply_min.x as usize, self.apply_min.y as usize, self.apply_min.z as usize), max: (self.apply_max.x as usize, self.apply_max.y as usize, self.apply_max.z as usize) }, self.steps as usize );
                        if res.is_err() {
                            return Err( format!("CellularAutomata command '{}' errored out with '{}'", name, res.err().unwrap() ) );
                        } else {
                            input.elements.insert( self.save.to_string(), res.unwrap() );
                            return Ok(input);
                        }
                    } else {
                        return Err( format!("Attempted to run CellularAutomata command '{}' on missing input!", name ) );
                    }
                } else {
                    return Err( format!("Attempted to run CellularAutomata command '{}' without a rule set!", name ) );
                }
            },
            CommandMode::IntervalSelect => {
                let mut select = Box::new( HashSet::<(i64, i64, i64)>::new() );
                let sz = input.size;
                for x in ((self.offset.x as usize)..sz.0).step_by( self.interval.z as usize ) {
                    for y in ((self.offset.y as usize)..sz.1).step_by( self.interval.z as usize ) {
                        for z in ((self.offset.z as usize)..sz.2).step_by( self.interval.z as usize ) {
                            select.insert( (x as i64, y as i64, z as i64) );
                        }
                    }
                }
                input.elements.insert( self.save.to_string(), GridElement::Sel(select) );
                return Ok(input);
            },
            CommandMode::SelectFall => {
                if let Some(GridElement::Sel(sel)) = input.elements.get( &self.source.to_string() ) {
                    if let Some(GridElement::Sel(wall)) = input.elements.get( &self.solid.to_string() ) {
                        let mut output = Box::new( HashSet::<(i64, i64, i64)>::new() );
                        for pos in sel.clone().into_iter() {
                            let mut prev = pos;
                            let mut fore = pos;
                            if self.sf_reverse {
                                fore = ( fore.0, fore.1 + 1, fore.2 );
                            } else {
                                fore = ( fore.0, fore.1 - 1, fore.2 );
                            }
                            while !wall.contains(&fore) {
                                if self.column {
                                    output.insert(prev);
                                }
                                prev = fore;
                                if self.sf_reverse {
                                    fore = ( fore.0, fore.1 + 1, fore.2 );
                                } else {
                                    fore = ( fore.0, fore.1 - 1, fore.2 );
                                }
                            }
                            output.insert(prev);
                        }
                        input.elements.insert( self.save.to_string(), GridElement::Sel(output) );
                        return Ok(input);
                    } else {
                        return Err( format!("Attempted to run SelectFall command '{}' with a non-boolean solid wall field!", name ) );
                    }
                } else {
                    return Err( format!("Attempted to run SelectFall command '{}' with a non-boolean source!", name ) );
                }
            },
            _ => { return Err( format!("Attempted to run command '{}' by providing one input, incorrectly!", name ) ); },
        }
    }


}

