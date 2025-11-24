use std::collections::{HashSet, HashMap};

use ndarray::{Array3, Axis};

use ultraviolet::vec::Vec3;

use grid_ray::GridRayIter3;
use grid_ray::ilattice::glam::{IVec3, Vec3A};

use godot::global::godot_print;

use crate::datagrid::Selection;



#[derive(Clone)]
struct Node {
    cost: f32,
    end_dist: f32,
    parent: [usize; 3],
    pvec: Vec3,
}

impl Node {
    fn score(&self) -> i64 {
        let fscore = self.cost + self.end_dist * 1.2;
        (fscore * 8192.0) as i64
    }

}

pub struct SearchMap {
    pub weight_array: Array3<f64>,
    pub max_slope: f32,
    pub vertical_skew: f32,
}

impl SearchMap {
    pub fn thstar( &self, startpos: (i64, i64, i64), endpos: (i64, i64, i64) ) -> Result<Selection, ()> {
        let mut open = HashMap::<[usize; 3], Node>::new();

        let start = [ startpos.0 as usize, startpos.1 as usize, startpos.2 as usize ];
        let end = [ endpos.0 as usize, endpos.1 as usize, endpos.2 as usize ];

        open.insert( start.clone(), Node{ cost: 0.0, end_dist: self.distance(&start, &end), parent: start.clone(), pvec: Vec3::zero() } );

        let mut closed = HashMap::<[usize; 3], Node>::new();

        let offset_list = [
            (0,0,1), (0,0,-1), (1,0,0), (-1,0,0),
            (0,1,1), (0,1,-1), (1,1,0), (-1,1,0),
            (0,-1,1), (0,-1,-1), (1,-1,0), (-1,-1,0)
        ];

        while !open.is_empty() {
            let key : [usize; 3];
            {
                key = *open.iter().min_by_key( | (k, v) | v.score() ).unwrap().0;
            }
            let best = open.remove_entry( &key );

            let Some((bp, bn)) = best else { return Err(()) };

            if bp == end {
                let mut sel = Box::new( HashSet::<(i64, i64, i64)>::new() );
                let mut current = bp;
                let mut node = &bn;
                while current != start {
                    if closed.contains_key(&current) {
                        node = closed.get(&current).unwrap();
                    }
                    let p = node.parent;
                    let v = node.pvec;
                    sel = self.search_select(p, v, sel);
                    current = p;
                }
                return Ok(sel);
            }

            for offs in offset_list {
                let (x,y,z) = offs;
                let neighbor = [ bp[0] as i64 + x, bp[1] as i64 + y, bp[2] as i64 + z ];
                let neighbor = self.check(neighbor);
                if neighbor.is_none() {continue;}
                let neighbor = neighbor.unwrap();
                if closed.contains_key(&neighbor) {continue;}

                let nvec = Vec3::new( neighbor[0] as f32 - bp[0] as f32, neighbor[1] as f32 - bp[1] as f32, neighbor[2] as f32 - bp[2] as f32 );

                let dx = ( nvec.x.powi(2) + nvec.z.powi(2) ).sqrt();
                let dotprod = nvec.normalized().dot( bn.pvec.normalized() );
                if nvec.y.abs() / dx.abs() > self.max_slope || dotprod > -0.05 {continue;}

                if !open.contains_key(&neighbor) {
                    open.insert( neighbor.clone(), Node{ cost: 1000000.0, end_dist: self.distance(&end, &neighbor), parent: bp, pvec: nvec } );
                }

                let mr = open.get_mut(&neighbor).unwrap();
                if dotprod < -0.75 {
                    let p = bn.parent;
                    let new_cost = closed.get(&p).unwrap().cost + self.search_cost( p, nvec );
                    if new_cost < mr.cost {
                        mr.cost = new_cost;
                        mr.parent = p;
                        mr.pvec = Vec3::new( neighbor[0] as f32 - p[0] as f32, neighbor[1] as f32 - p[1] as f32, neighbor[2] as f32 - p[2] as f32 );
                    }
                } else {
                    let new_cost = bn.cost + self.search_cost( bp, nvec );
                    if new_cost < mr.cost {
                        mr.cost = new_cost;
                        mr.parent = bp;
                        mr.pvec = nvec;
                    }
                }



            }

            closed.insert( bp, bn );

        }

        return Err(());
    }

    pub fn search_cost( &self, start: [usize; 3], along: Vec3 ) -> f32 {
        let mut cost = 0.0;
        let mut traversal = GridRayIter3::new( Vec3A::from_array([ start[0] as f32, start[1] as f32, start[2] as f32 ]), Vec3A::from_array([along.x, along.y, along.z]) );
        let mut mag = along.mag();
        let mut et = 0.0;
        while et < mag {
            let next = traversal.next().unwrap();
            if et > 0.0 {
                let ch = self.check( [next.1.x as i64, next.1.y as i64, next.1.z as i64] );
                if ch.is_none() { break; }
                cost += self.weight_array[ ch.unwrap() ];
            }
            et = next.0;
        }
        cost as f32
    }

    pub fn search_select( &self, start: [usize; 3], along: Vec3, mut sel: Selection ) -> Selection {
        let mut cost = 0.0;
        let mut traversal = GridRayIter3::new( Vec3A::from_array([ start[0] as f32, start[1] as f32, start[2] as f32 ]), Vec3A::from_array([along.x, along.y, along.z]) );
        let mut mag = along.mag();
        let mut et = 0.0;
        while et < mag {
            let next = traversal.next().unwrap();
            sel.insert( (next.1.x as i64, next.1.y as i64, next.1.z as i64) );
            et = next.0;
        }
        sel
    }

    pub fn check( &self, a: [i64; 3] ) -> Option<[usize; 3]> {
        let dim = self.weight_array.dim();
        let dim = ( dim.0 as i64, dim.1 as i64, dim.2 as i64 );
        if a[0] < 0 || a[0] >= dim.0 {
            return None;
        } else if a[1] < 0 || a[1] >= dim.1 {
            return None;
        } else if a[2] < 0 || a[2] >= dim.2 {
            return None;
        }
        Some([ a[0] as usize, a[1] as usize, a[2] as usize ])
    }

    pub fn distance( &self, a: &[usize; 3], b: &[usize; 3] ) -> f32 {
        let mid = ( a[0] as f32 - b[0] as f32 ).powi(2) + ( a[2] as f32 - b[2] as f32 ).powi(2);
        ( mid.sqrt() + self.vertical_skew.powi(2) * ( a[1] as f32 - b[1] as f32 ).powi(2) ).sqrt()
    }
}
