
use godot::prelude::*;
use godot::classes::{GridMap};

use godot::global::{godot_warn, godot_error};

use ndarray::Array3;

use crate::resource::{MapGenCommand, NeedsInput};
use crate::datagrid::{DataGrid, GridElement};


#[derive(GodotClass)]
#[class(tool, init, base=GridMap)]
pub struct GeneratedGridMap {
    base: Base<GridMap>,
    #[export]
    pub editor_placement_offset: Vector3i,
    #[export]
    pub editor_seed: i64,
    pub result_grid: Option<DataGrid>,
}

#[godot_api]
impl GeneratedGridMap {

    #[func]
    pub fn generate_default(&mut self) {
        self.generate( self.editor_seed );
    }

    #[func]
    pub fn generate( &mut self, seed: i64 ) {
        let ch = self.base().get_child(0);
        if ch.is_none() {
            godot_error!("GeneratedGridMap node must have a MapGenNode as its first child!");
            return;
        }
        let ch = ch.unwrap();

        let as_mgn = ch.try_cast::<MapGenNode>();
        if as_mgn.is_err() {
            godot_error!("GeneratedGridMap node must have a MapGenNode as its first child!");
            return;
        }
        let as_mgn = as_mgn.unwrap();

        let gen_result = as_mgn.bind().generate( seed );

        if gen_result.is_err() {
            godot_error!("GeneratedGridMap encountered error:\n{}", gen_result.err().unwrap() );
            self.result_grid = None;
            return;
        }

        self.result_grid = gen_result.ok();
        self.signals().generation_finished().emit();
    }

    #[signal]
    pub fn generation_finished();

    #[func]
    pub fn place_default(&mut self) {
        self.place( self.editor_placement_offset );
    }

    #[func]
    pub fn place( &mut self, offset: Vector3i ) {
        if self.result_grid.is_none() {
            godot_error!("GeneratedGridMap node has no stored data grid - have you run the generate command successfully?");
            return;
        }

        let rg = self.result_grid.take().unwrap();

        let block = rg.elements.get("mesh");
        if block.is_none() {
            godot_error!("GeneratedGridMap node couldn't place meshes: no 'mesh' integer field found on data grid.");
            return;
        }
        let block = block.unwrap();

        let arr : &Array3<i64>;

        if let GridElement::Int( iarr ) = block {
            arr = iarr;
        } else {
            godot_error!("GeneratedGridMap node couldn't place meshes: 'mesh' field on data grid wasn't integer type.");
            return;
        }

        let mut rotation_arr : Option<&Array3<i64>> = None;

        let rot = rg.elements.get("rotation");
        if rot.is_some() {
            let rot = rot.unwrap();

            if let GridElement::Int( iarr ) = rot {
                rotation_arr = Some(iarr);
            } else {
                godot_warn!("GeneratedGridMap node found a 'rotation' field on data grid, but it wasn't integer type. Meshes will be placed without rotation.");
            }
        }

        for x in 0..rg.size.0 {
            for y in 0..rg.size.1 {
                for z in 0..rg.size.2 {
                    let mut b = self.base_mut();
                    let mut ex = b.set_cell_item_ex( offset + Vector3i::new(x as i32, y as i32, z as i32), arr[[x, y, z]] as i32 );
                    if let Some(rot) = rotation_arr {
                        ex = ex.orientation( rot[[x, y, z]] as i32 );
                    }
                    ex.done();
                }
            }
        }

        self.result_grid = Some(rg);
    }

    #[func]
    pub fn sample_grid(&self, name: GString, position: Vector3i ) -> Variant {
        if let Some(ref rg) = self.result_grid {
            if let Some(ref ge) = rg.elements.get(&name.to_string()) {
                match ge {
                    GridElement::Int(arr) => { return arr[[ position.x as usize, position.y as usize, position.z as usize ]].to_variant() },
                    GridElement::Float(arr) => { return arr[[ position.x as usize, position.y as usize, position.z as usize ]].to_variant() },
                    GridElement::Sel(select) => { return select.contains( &(position.x as i64, position.y as i64, position.z as i64) ).to_variant() },
                    GridElement::Rooms(vec) => {
                        for (idx, rm) in vec.into_iter().enumerate() {
                            if rm.members.contains( &(position.x as i64, position.y as i64, position.z as i64) ) {
                                return (idx as i64).to_variant();
                            }
                        }
                        return (-1).to_variant();
                    },
                    GridElement::List(vec) => {
                        for (idx, pos) in vec.into_iter().enumerate() {
                            if (position.x as i64, position.y as i64, position.z as i64) == *pos {
                                return (idx as i64).to_variant();
                            }
                        }
                        return (-1).to_variant();
                    },
                }
            } else {
                godot_error!("Attempt to sample GeneratedGridMap failed due to nonexistent field name.");
            }
        } else {
            godot_error!("Attempt to sample GeneratedGridMap failed due to lack of successfully generated data grid.");
        }
        return 0.to_variant();
    }

    #[func]
    pub fn get_fields(&self) -> Array<GString> {
        if let Some(ref rg) = self.result_grid {
            return Array::from( rg.elements.keys().map( |st| GString::from(st) ).collect::<Vec<_>>().as_slice() );
        } else {
            godot_error!("Attempt to get list on GeneratedGridMap failed due to lack of successfully generated data grid.");
            return Array::new();
        }
    }

    #[func]
    pub fn get_list(&self, name: GString) -> Array<Vector3i> {
        if let Some(ref rg) = self.result_grid {
            if let Some(GridElement::List( ls )) = rg.elements.get(&name.to_string()) {
                return Array::from( ls.into_iter().map( | tup | Vector3i::new( tup.0 as i32, tup.1 as i32, tup.2 as i32 ) ).collect::<Vec<_>>().as_slice() );
            } else {
                godot_error!("Attempt to get list on GeneratedGridMap failed due to missing or incorrect-typed list.");
                return Array::new();
            }
        } else {
            godot_error!("Attempt to get list on GeneratedGridMap failed due to lack of successfully generated data grid.");
            return Array::new();
        }
    }

}



#[derive(GodotClass)]
#[class(tool, init, base=Node)]
pub struct MapGenNode {
    base: Base<Node>,
    #[export]
    pub command: Option<Gd<MapGenCommand>>,
}

#[godot_api]
impl MapGenNode {
    pub fn generate( &self, seed: i64 ) -> Result<DataGrid, String> {
        if self.command.is_none() {
            return Err( "No command resource set in a generation node!".to_string() );
        }

        let comm = self.command.as_ref().unwrap();
        let needsinput = comm.bind().needs_input();

        if needsinput == NeedsInput::No {
            return comm.bind().run_none( seed, self.base().get_name().to_string() );
        }

        if needsinput == NeedsInput::One {
            let ch = self.base().get_child(0);
            if ch.is_none() {
                return Err( "MapGenNode configured to need a child MapGenNode found none!".to_string() );
            }
            let ch = ch.unwrap();

            let as_mgn = ch.try_cast::<MapGenNode>();
            if as_mgn.is_err() {
                return Err( "MapGenNode configured to need a child MapGenNode found none!".to_string() );
            }
            let as_mgn = as_mgn.unwrap();

            let gen_result = as_mgn.bind().generate( seed );

            if gen_result.is_err() {
                return gen_result;
            } else {
                return comm.bind().run_one( seed, gen_result.unwrap(), self.base().get_name().to_string() );
            }
        }


        return Err( "Unknown MapGenCommand configuration!".to_string() );

    }
}
