use std::collections::HashSet;

use godot::prelude::*;
use godot::classes::FastNoiseLite;

use ndarray::{Array, Array3};

use crate::datagrid::{DataGrid, GridElement, ElemType, Selection};
use crate::algorithm::{AlgorithmHelper, RectPrism};


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
    RandomRooms,
}

#[derive(GodotConvert, Var, Export, Default, PartialEq, Eq, Clone, Copy)]
#[godot(via = i64)]
pub enum EdgeMode {
    #[default]
    Ignore,
    Loop,
    Clamp
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
    #[export]
    pub nh_source: GString,

    #[export_group(name = "DropFields mode")]
    #[export]
    pub to_drop: godot::prelude::Array<GString>,

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
            CommandMode::Expressions => NeedsInput::One,
            CommandMode::SampleNoise => NeedsInput::One,
            CommandMode::SampleNeighborhood => NeedsInput::One,
            CommandMode::OuterWalls => NeedsInput::One,
            CommandMode::DropFields => NeedsInput::One,
            CommandMode::RandomRooms => NeedsInput::One,
        }
    }
}

impl MapGenCommand {

    pub fn run_none( &self, seed: i64 ) -> Result<DataGrid, String> {
        match self.mode {
            CommandMode::Initialize => {
                if self.init_size.x < 1 || self.init_size.y < 1 || self.init_size.z < 1 {
                    return Err( format!("Initialize command '{}' was set to invalid size!", self.base().get_name() ) );
                }
                return Ok( DataGrid::sized( ( self.init_size.x as usize, self.init_size.y as usize, self.init_size.z as usize ) ) );
            },
            _ => { return Err( format!("Attempted to run command '{}' by without input, but input was expected!", self.base().get_name() ) ); },
        }
    }

    pub fn run_one( &self, seed: i64, mut input: DataGrid ) -> Result<DataGrid, String> {
        match self.mode {
            CommandMode::Expressions => {
                for e in self.expression_list.iter_shared() {
                    let e = e.bind();

                    let res = input.parallel_expr( e.name.to_string(), &e.expr, &e.result_kind );
                    if res.is_err() {
                        return Err( format!("Error running command '{}': Expression '{}' execution failed with '{:?}'", self.base().get_name(), e.name, res.unwrap_err() ) );
                    }
                }
                return Ok(input);

            },
            CommandMode::SampleNoise => {
                if self.noise.is_none() {
                    return Err( format!("SampleNoise command '{}' had no noise supplied!", self.base().get_name() ) );
                }
                let mut noise = self.noise.clone().unwrap();
                noise.set_seed( (seed + self.seed_salt) as i32 );
                let sample = Array::from_shape_fn(input.size, | (x, y, z) | {noise.get_noise_3d( x as f32, y as f32, z as f32 ) as f64} );
                input.elements.insert( self.save.to_string(), GridElement::Float(sample) );
                return Ok(input);

            },
            CommandMode::SampleNeighborhood => {
                if self.neighborhood.is_none() {
                    return Err( format!("SampleNeighborhood command '{}' had no neighborhood supplied!", self.base().get_name() ) );
                }
                let res = input.sample_neighborhood( self.neighborhood.as_ref().unwrap(), self.edge_mode, &self.nh_source.to_string(), &self.save.to_string() );
                if res.is_err() {
                    return Err( format!("Error running command '{}': Sampling execution failed with '{:?}'", self.base().get_name(), res.unwrap_err() ) );
                }
                return Ok(input);

            },
            CommandMode::OuterWalls => {
                if self.save.is_empty() {
                    return Err( format!("OuterFaces command '{}' had empty save string supplied!", self.base().get_name() ) );
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
                    input.elements.remove( &f.to_string() );
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
                    return Err( format!("Error running command '{}': Random rooms failed with '{:?}'", self.base().get_name(), res.err().unwrap() ) );
                }
            },
            _ => { return Err( format!("Attempted to run command '{}' by providing one input, incorrectly!", self.base().get_name() ) ); },
        }
    }


}


