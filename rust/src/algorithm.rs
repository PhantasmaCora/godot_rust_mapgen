use std::collections::HashSet;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

//use ndarray::{Array, Array3};

use crate::datagrid::{DataGrid, GridElement, ElemType, Selection, Room};




pub struct RectPrism {
    pub min: (usize, usize, usize),
    pub max: (usize, usize, usize)
}

pub struct AlgorithmHelper {}

impl AlgorithmHelper {
    pub fn random_rooms( count: i64, seed: i64, within: RectPrism, sized: RectPrism, allow_overlap: bool ) -> Result<( Vec<Room>, Selection), String> {
        let mut rms = Vec::<Room>::new();
        let mut uni = Box::new( HashSet::<(i64, i64, i64)>::new() );
        let mut random = ChaCha12Rng::seed_from_u64( seed as u64 );

        let mut safety = count * 2;

        while safety > 0 && rms.len() < count as usize {
            let sx = random.random_range( sized.min.0..=sized.max.0 );
            let sy = random.random_range( sized.min.1..=sized.max.1 );
            let sz = random.random_range( sized.min.2..=sized.max.2 );

            let px = random.random_range( within.min.0..(within.max.0 - sx) );
            let py = random.random_range( within.min.1..(within.max.1 - sy) );
            let pz = random.random_range( within.min.2..(within.max.2 - sz) );

            let center = ( (px + sx / 2) as i64, (py + sy / 2) as i64,  (pz + sz / 2) as i64 );
            let mut members = Box::new( HashSet::<(i64, i64, i64)>::new() );

            for x in 0..sx {
                for y in 0..sy {
                    for z in 0..sz {
                        members.insert( ( (x + px) as i64, (y + py) as i64, (z + pz) as i64 ) );
                    }
                }
            }

            if allow_overlap || uni.is_disjoint(&members) {
                uni = Box::new( &*uni | &*members );
                rms.push( Room{ members, center: Some(center) } );
            }
        }

        return Ok( (rms, uni) );
    }

}
