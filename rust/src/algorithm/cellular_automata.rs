

use ndarray::Array3;

use godot::prelude::*;
use godot::classes::Expression;
use godot::builtin::array;

use crate::datagrid::{GridElement, Helper};
use crate::resource::Neighborhood;
use crate::algorithm::RectPrism;



#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct CellAutoRule {
    base: Base<Resource>,
    #[export]
    neighborhood: Option<Gd<Neighborhood>>,
    #[export]
    result_expr: GString,
    #[export]
    provide_randomness: bool,
}

impl CellAutoRule {
    pub fn run(&self, data: GridElement, area: RectPrism, steps: usize) -> Result<GridElement, String> {
        if let GridElement::Int( mut array ) = data {
            for _i in 0..steps {
                let res = self.run_istep( array, area.clone() );
                if res.is_err() { return Err(res.err().unwrap()); }
                array = res.unwrap();
            }
            return Ok( GridElement::Int(array) );
        } else if let GridElement::Float( mut array ) = data {
            for _i in 0..steps {
                let res = self.run_fstep( array, area.clone() );
                if res.is_err() { return Err(res.err().unwrap()); }
                array = res.unwrap();
            }
            return Ok( GridElement::Float(array) );
        } else {
            return Err( "Invalid format for running CA on, must be Int or Float".to_string() );
        }
    }

    fn run_istep(&self, mut array: Array3<i64>, area: RectPrism) -> Result<Array3<i64>, String> {
        let help = Helper::new_alloc();

        let nh = self.neighborhood.as_ref().unwrap().bind();

        let mut expression = Expression::new_gd();
        let pres : godot::global::Error;
        if nh.summing_expr.is_empty() {
            pres = expression.parse_ex( &GString::from( "acc + this" ) ).input_names( &PackedStringArray::from( [ GString::from("acc"), GString::from("this") ] ) ).done();
        } else {
            pres = expression.parse_ex( &nh.summing_expr ).input_names( &PackedStringArray::from( [ GString::from("acc"), GString::from("this") ] ) ).done();
        }

        if !( pres == godot::global::Error::OK ) {
            return Err( format!( "Expression parse error: {:?}", pres ) );
        }

        let mut res_expression = Expression::new_gd();
        let pres : godot::global::Error;
        if self.result_expr.is_empty() {
            pres = res_expression.parse_ex( &GString::from( "sum" ) ).input_names( &PackedStringArray::from( [GString::from("state"), GString::from("sum") ] ) ).done();
        } else {
            pres = res_expression.parse_ex( &self.result_expr ).input_names( &PackedStringArray::from( [GString::from("state"), GString::from("sum") ] ) ).done();
        }

        if !( pres == godot::global::Error::OK ) {
            return Err( format!( "Result expression parse error: {:?}", pres ) );
        }

        for x in area.min.0..area.max.0 {
            for y in area.min.1..area.max.1 {
                for z in area.min.2..area.max.2 {
                    let mut accumulator = nh.accumulator_base as i64;

                    for os in nh.offsets.iter_shared() {
                        let checkpos = self.check_pos( ( x as i32 + os.x, y as i32 + os.y, z as i32 + os.z ) );
                        if checkpos.is_none() {
                            continue;
                        }
                        let checkpos = checkpos.unwrap();

                        let vari = expression.execute_ex().inputs( &array![ &accumulator.to_variant(), &array[checkpos].to_variant() ] ).base_instance( &help.clone().upcast::<Object>() ).done();
                        accumulator = vari.to::<i64>();
                    }

                    let svari = res_expression.execute_ex().inputs( &array![ &array[[ x as usize, y as usize, z as usize ]].to_variant(), &accumulator.to_variant() ] ).base_instance( &help.clone().upcast::<Object>() ).done();
                    array[[ x as usize, y as usize, z as usize ]] = svari.to::<i64>();

                }
            }
        }

        Ok(array)
    }


    fn run_fstep(&self, mut array: Array3<f64>, area: RectPrism) -> Result<Array3<f64>, String> {
        let help = Helper::new_alloc();

        let nh = self.neighborhood.as_ref().unwrap().bind();

        let mut expression = Expression::new_gd();
        let pres : godot::global::Error;
        if nh.summing_expr.is_empty() {
            pres = expression.parse_ex( &GString::from( "acc + this" ) ).input_names( &PackedStringArray::from( [ GString::from("acc"), GString::from("this") ] ) ).done();
        } else {
            pres = expression.parse_ex( &nh.summing_expr ).input_names( &PackedStringArray::from( [ GString::from("acc"), GString::from("this") ] ) ).done();
        }

        if !( pres == godot::global::Error::OK ) {
            return Err( format!( "Expression parse error: {:?}", pres ) );
        }

        let mut res_expression = Expression::new_gd();
        let pres : godot::global::Error;
        if self.result_expr.is_empty() {
            pres = res_expression.parse_ex( &GString::from( "sum" ) ).input_names( &PackedStringArray::from( [GString::from("state"), GString::from("sum") ] ) ).done();
        } else {
            pres = res_expression.parse_ex( &self.result_expr ).input_names( &PackedStringArray::from( [GString::from("state"), GString::from("sum") ] ) ).done();
        }

        if !( pres == godot::global::Error::OK ) {
            return Err( format!( "Result expression parse error: {:?}", pres ) );
        }

        for x in area.min.0..area.max.0 {
            for y in area.min.1..area.max.1 {
                for z in area.min.2..area.max.2 {
                    let mut accumulator = nh.accumulator_base;

                    for os in nh.offsets.iter_shared() {
                        let checkpos = self.check_pos( ( x as i32 + os.x, y as i32 + os.y, z as i32 + os.z ) );
                        if checkpos.is_none() {
                            continue;
                        }
                        let checkpos = checkpos.unwrap();

                        let vari = expression.execute_ex().inputs( &array![ &accumulator.to_variant(), &array[checkpos].to_variant() ] ).base_instance( &help.clone().upcast::<Object>() ).done();
                        accumulator = vari.to::<f64>();
                    }

                    let svari = res_expression.execute_ex().inputs( &array![ &array[[ x as usize, y as usize, z as usize ]].to_variant(), &accumulator.to_variant() ] ).base_instance( &help.clone().upcast::<Object>() ).done();
                    array[[ x as usize, y as usize, z as usize ]] = svari.to::<f64>();

                }
            }
        }

        Ok(array)
    }

    // should probably have safety checking eventually but it'll do for now.'
    fn check_pos( &self, pos: (i32, i32, i32) ) -> Option<[usize; 3]> {
        Some([pos.0 as usize, pos.1 as usize, pos.2 as usize])
    }

}
