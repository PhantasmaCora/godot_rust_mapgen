use std::collections::{HashMap, HashSet};

use godot::prelude::*;
use godot::classes::Expression;
use godot::builtin::array;

use ndarray::Array3;

use crate::resource::{Neighborhood, EdgeMode};




pub type Selection = Box<HashSet<(i64, i64, i64)>>;


pub enum GridElement {
    Int( Array3<i64> ),
    Float( Array3<f64> ),
    Sel( Selection ),
    Rooms( Vec<Room> ),
}

#[derive(GodotConvert, Var, Export, Default)]
#[godot(via = i64)]
pub enum ElemType {
    Int,
    #[default]
    Float,
    Bool
}

pub struct Room {
    pub members: Selection,
    pub center: Option<(i64, i64, i64)>,
}

pub struct DataGrid {
    pub size: (usize, usize, usize),
    pub elements: HashMap<String, GridElement>,
}

impl DataGrid {
    pub fn sized( sz: (usize, usize, usize) ) -> Self {
        Self {
            size: sz,
            elements: HashMap::<String, GridElement>::new(),
        }
    }

    pub fn parallel_expr(&mut self, name: String, expr: &GString, typ: &ElemType) -> Result<(), String> {
        let help = Helper::new_alloc();

        let mut new_ge : GridElement;

        match typ {
            ElemType::Int => { new_ge = GridElement::Int( Array3::<i64>::zeros(self.size) ); },
            ElemType::Float => { new_ge = GridElement::Float( Array3::<f64>::zeros(self.size) ); },
            ElemType::Bool => { new_ge = GridElement::Sel( Box::new( HashSet::<(i64, i64, i64)>::new() ) ); }
        }

        let mut expression = Expression::new_gd();
        let pres = expression.parse_ex( expr ).input_names( &PackedStringArray::from( [ GString::from("dict") ] ) ).done();

        if !( pres == godot::global::Error::OK ) {
            return Err( format!( "Expression parse error: {:?}", pres ) );
        }

        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                for z in 0 .. self.size.2 {
                    let mut dict = Dictionary::new();
                    dict.insert( "position", Vector3i::new( x as i32, y as i32, z as i32 ) );

                    for (k, v) in &self.elements {
                        if let GridElement::Int( arr ) = v {
                            dict.insert( GString::from(k), arr[[x, y, z]] );
                        } else if let GridElement::Float( arr ) = v {
                            dict.insert( GString::from(k), arr[[x, y, z]] );
                        } else if let GridElement::Sel( select ) = v {
                            dict.insert( GString::from(k), select.contains( &( x as i64, y as i64, z as i64 )) );
                        }
                    }

                    let vari = expression.execute_ex().inputs( &array![ &dict.to_variant() ] ).base_instance( &help.clone().upcast::<Object>() ).done();

                    if let GridElement::Int( ref mut arr ) = new_ge {
                        let val = vari.to::<i64>();
                        arr[[x, y, z]] = val;
                    } else if let GridElement::Float( ref mut arr ) = new_ge {
                        let val = vari.to::<f64>();
                        arr[[x, y, z]] = val;
                    } else if let GridElement::Sel( ref mut select ) = new_ge {
                        let val = vari.to::<bool>();
                        if val {
                            select.insert( (x as i64, y as i64, z as i64) );
                        }
                    }
                }
            }
        }

        self.elements.insert(name, new_ge);

        help.free();

        return Ok(());
    }

    pub fn sample_neighborhood(&mut self, nh: &Gd<Neighborhood>, em: EdgeMode, source: &str, save: &str) -> Result<(), String> {
        let help = Helper::new_alloc();

        let nh = nh.bind();

        let source_elem = self.elements.get( source );

        if source_elem.is_none() {
            return Err( format!( "Field '{}' not found on data grid.", source ) );
        }
        let source_elem = source_elem.unwrap();


        let mut new_ge : GridElement;

        let mut is_bool = false;

        match source_elem {
            GridElement::Int(_) => { new_ge = GridElement::Int( Array3::<i64>::zeros(self.size) ); },
            GridElement::Float(_) => { new_ge = GridElement::Float( Array3::<f64>::zeros(self.size) ); },
            GridElement::Sel(_) => { is_bool = true; new_ge = GridElement::Sel( Box::new( HashSet::<(i64, i64, i64)>::new() ) ); },
            GridElement::Rooms(_) => { return Err( "SampleNeighborhood called on a room list field (incompatible).".to_string() ) },
        }

        let mut expression = Expression::new_gd();
        let pres : godot::global::Error;
        if nh.summing_expr.is_empty() {
            pres = expression.parse_ex( &GString::from( if is_bool { "acc || this" } else { "acc + this" } ) ).input_names( &PackedStringArray::from( [ GString::from("acc"), GString::from("this") ] ) ).done();
        } else {
            pres = expression.parse_ex( &nh.summing_expr ).input_names( &PackedStringArray::from( [ GString::from("acc"), GString::from("this") ] ) ).done();
        }

        if !( pres == godot::global::Error::OK ) {
            return Err( format!( "Expression parse error: {:?}", pres ) );
        }

        if let GridElement::Int( ref mut arr ) = new_ge {

            for x in 0..(self.size.0 as i32) {
                for y in 0..(self.size.1 as i32) {
                    for z in 0 .. (self.size.2 as i32) {
                        let mut accumulator = nh.accumulator_base as i64;

                        for os in nh.offsets.iter_shared() {
                            let checkpos = self.check_pos( ( x + os.x, y + os.y, z + os.z ), em );
                            if checkpos.is_none() {
                                continue;
                            }
                            let checkpos = checkpos.unwrap();

                            let GridElement::Int( arr ) = source_elem else { return Err("Should be unreachable".to_string()); };
                            let vari = expression.execute_ex().inputs( &array![ &accumulator.to_variant(), &arr[checkpos].to_variant() ] ).base_instance( &help.clone().upcast::<Object>() ).done();
                            accumulator = vari.to::<i64>();
                        }

                        arr[[ x as usize, y as usize, z as usize ]] = accumulator;

                    }
                }
            }
        } else if let GridElement::Float( ref mut arr ) = new_ge {

            for x in 0..(self.size.0 as i32) {
                for y in 0..(self.size.1 as i32) {
                    for z in 0 .. (self.size.2 as i32) {
                        let mut accumulator = nh.accumulator_base;

                        for os in nh.offsets.iter_shared() {
                            let checkpos = self.check_pos( ( x + os.x, y + os.y, z + os.z ), em );
                            if checkpos.is_none() {
                                continue;
                            }
                            let checkpos = checkpos.unwrap();

                            let GridElement::Float( arr ) = source_elem else { return Err("Should be unreachable".to_string()); };
                            let vari = expression.execute_ex().inputs( &array![ &accumulator.to_variant(), &arr[checkpos].to_variant() ] ).base_instance( &help.clone().upcast::<Object>() ).done();
                            accumulator = vari.to::<f64>();
                        }

                        arr[[ x as usize, y as usize, z as usize ]] = accumulator;
                    }
                }
            }
        } else if let GridElement::Sel( ref mut select ) = new_ge {

            for x in 0..(self.size.0 as i32) {
                for y in 0..(self.size.1 as i32) {
                    for z in 0 .. (self.size.2 as i32) {
                        let mut accumulator = nh.accumulator_base > 0.0;

                        for os in nh.offsets.iter_shared() {
                            let checkpos = self.check_pos( ( x + os.x, y + os.y, z + os.z ), em );
                            if checkpos.is_none() {
                                continue;
                            }
                            let checkpos = checkpos.unwrap();

                            let GridElement::Sel( select ) = source_elem else { return Err("Should be unreachable".to_string()); };
                            let vari = expression.execute_ex().inputs( &array![ &accumulator.to_variant(), &select.contains( &(checkpos[0] as i64, checkpos[1] as i64, checkpos[2] as i64) ).to_variant() ] ).base_instance( &help.clone().upcast::<Object>() ).done();
                            accumulator = vari.to::<bool>();
                        }

                        if accumulator {
                            select.insert( (x as i64, y as i64, z as i64) );
                        }
                    }
                }
            }
        }

        self.elements.insert( save.to_string(), new_ge );

        help.free();

        Ok(())
    }

    fn check_pos( &self, pos: (i32, i32, i32), mode: EdgeMode ) -> Option<[usize; 3]> {

        let newx : usize;
        if pos.0 > 0 && pos.0 < self.size.0 as i32 {
            newx = pos.0 as usize;
        } else if mode == EdgeMode::Ignore {
            return None;
        } else if mode == EdgeMode::Loop {
            newx = ( pos.0 % ( self.size.0 as i32 ) ) as usize;
        } else if pos.0 < 0 {
            newx = 0;
        } else {
            newx = self.size.0 - 1;
        }

        let newy : usize;
        if pos.1 > 0 && pos.1 < self.size.1 as i32 {
            newy = pos.1 as usize;
        } else if mode == EdgeMode::Ignore {
            return None;
        } else if mode == EdgeMode::Loop {
            newy = ( pos.1 % ( self.size.1 as i32 ) ) as usize;
        } else if pos.1 < 0 {
            newy = 0;
        } else {
            newy = self.size.1 - 1;
        }

        let newz : usize;
        if pos.2 > 0 && pos.2 < self.size.2 as i32 {
            newz = pos.2 as usize;
        } else if mode == EdgeMode::Ignore {
            return None;
        } else if mode == EdgeMode::Loop {
            newz = ( pos.2 % ( self.size.2 as i32 ) ) as usize;
        } else if pos.2 < 0 {
            newz = 0;
        } else {
            newz = self.size.2 - 1;
        }

        Some( [newx, newy, newz] )
    }
}

#[derive(GodotClass)]
#[class(tool, init, base=Object)]
struct Helper {}

#[godot_api]
impl Helper {
    #[func]
    pub fn ternary(p_if: bool, p_then: Variant, p_else: Variant) -> Variant {
        if p_if {
            return p_then
        } else {
            return p_else
        }
    }
}
