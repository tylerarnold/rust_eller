
#![allow(unused_variables, dead_code)]



//use rand::random;

use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;
use std::collections::HashMap;
use rand::random;


#[derive(Hash, PartialEq, Eq, Debug, Clone)]
enum Wall {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug,Clone)]
pub struct Cell {
    set_id: usize, 
    walls: HashSet<Wall>,   
}

impl Cell {
    fn new(set_id: usize) -> Self {
        let h = HashSet::from([Wall::Left, Wall::Right, Wall::Top, Wall::Bottom]);
        let cell = Cell {
            walls: h, // in C, I would use a bitmap, curious about how this compares. 
            set_id:set_id, 
        };
        cell
    }
}

#[derive(Debug,Clone)]
pub struct Row {
    id: usize, // row number
    width: usize, // row width
    cells: Vec<Rc<RefCell<Cell>>>,
}
impl Row {
    fn new(width:usize, id: usize) -> Self {
        let mut row = Row {
            id: id,
            width: width,
            cells: Vec::new(),
        };
        for id in id..id+width {
            row.cells.push(Rc::new(RefCell::new(Cell::new(0))));
        }
        row
    }
}

//do not have to keep the entire maze
#[derive(Debug,Clone)]
pub struct Maze {
    width: usize,  // row width
    height: usize, // number of rows
    rows: Vec<Row>,
}

//
// for now crank out the maze until we get the visualization and algorithm correct
//
impl Maze  {
    fn new (width:usize,height:usize)->Self {
        let mut m = Maze {
            width: width,
            height: height,
            rows: Vec::new(),
        };
        for ri in 0..height {
            let row = Row::new(width,ri);
            m.rows.push(row);    
        }
        m
    }

    // for any zero member, create a new id 
    fn fill_row(start_id:usize, row: &Row) {
        let mut cur_id = start_id;
        for c in &row.cells {
            if c.borrow().set_id == 0 {
                c.borrow_mut().set_id = cur_id;
                cur_id = cur_id + 1;    
            }
        }
    }

    //
    // we randomly join adjacent cells that belong to different sets. 
    // I don't see the point in creating a hash or anything for this, it is a walk 
    // with a join coin toss. 
    //
    fn join_row_cells(row: &Row){
        
        //let first = row.cells.iter();
        //let second = row.cells.iter().skip(1);
        //first.zip(second).map(|(a,mut b)| if random() { b = a;} );
        for i in 0..row.cells.len()-1{
            if random() {
                if row.cells[i+1].borrow_mut().set_id != row.cells[i].borrow_mut().set_id {
                    row.cells[i+1].borrow_mut().set_id = row.cells[i].borrow_mut().set_id;
                    row.cells[i+1].borrow_mut().walls.remove(&Wall::Left);
                    row.cells[i].borrow_mut().walls.remove(&Wall::Right);
                }
            }
        }

    }

    fn join_last_row_cells(row: &Row) {
        for i in 0..row.cells.len()-1{
            row.cells[i+1].borrow_mut().set_id = row.cells[i].borrow_mut().set_id;
            row.cells[i+1].borrow_mut().walls.remove(&Wall::Left);
            row.cells[i].borrow_mut().walls.remove(&Wall::Right);
        }
    }
    
    fn punch_down(top: &Row, bottom: &Row,k:usize, s: usize){
        print!("\tdrop down , key {} start {} \n",k,s);
        
        //remove bottom wall of top cell
        top.cells[s].borrow_mut().walls.remove(&Wall::Bottom);
        
        //update bottom cell set_id and top wall
        let c = &bottom.cells[s];
        let bottom_set_id = c.borrow_mut().set_id;
        bottom.cells[s].borrow_mut().walls.remove(&Wall::Top);
        bottom.cells[s].borrow_mut().set_id = k; 

    }

    //Now we randomly determine the vertical connections, at least one per set. 
    // The cells in the next row that we connected to must be assigned to the set of the cell above them:
    fn join_rows(top: &Row, bottom: &Row){
        

        print!("\n join_rows \n");
        let mut map = HashMap::new();
        // store tuple with count and first index 
        // determine set sizes for this row 
        for (i,c) in top.cells.iter().enumerate() {
             map
            .entry(c.borrow_mut().set_id)
            .and_modify(|(s,c)| *c += 1)
            .or_insert((i,1));
        }

        //print!("{:?}\n",map);
        //for (k,t) in map.iter_mut().sorted_by_key(|k| k) {
        for (k,t) in map.iter_mut() {
             print!("{:?} {:?}\n",k,t);
             if t.1 == 1 {
                //size of one must punch down                
                let s = t.0;
                Self::punch_down(top, bottom,*k, s);

             } else {
                //coin flip drop down at least once 
                let s = t.0;
                let c = t.1;
                //create a vector of possible punches
                let mut punch_vec = Vec::new();
                for i in s..s+c {
                    if random() {
                        punch_vec.push(i);
                    }
                }
                if punch_vec.len() == 0 {
                    let s = rand::thread_rng().gen_range(1..c);
                    punch_vec.push(s);
                }
                for s in punch_vec {
                    Self::punch_down(top, bottom,*k,s);
                }

             }   

        }
        // now fill in remaining cells of next row and randomly join
        Self::fill_row(bottom.id*bottom.width+1,bottom);
        
        Self::join_row_cells(bottom);
    }

    pub fn construct(&mut self) {
        // init the first row 
        Self::fill_row(1,&self.rows[0]);
        
        Self::join_row_cells(&self.rows[0]);    

        for i in 0..self.rows.len()-1{
            Self::join_rows(&self.rows[i],&self.rows[i+1])
        }
        Self::join_last_row_cells(&self.rows[self.rows.len()-1]);
    }
    
    //  https://stackoverflow.com/questions/26952565/print-out-an-ascii-line-of-boxes
    const CELLWIDTHCHAR: usize = 12;
    const CELLHEIGHTCHAR: usize = 5;  

    //
    //   all rows are drawn as "benches" , except the last one. 
    //   +--------+
    //   |        | 

    fn print_horizontal(&self,row: &Row, is_last: bool) {

        if is_last {
            for i in 0..(Self::CELLWIDTHCHAR*(self.width)) {
                if i % Self::CELLWIDTHCHAR == 0 {
                    print!("+"); 
                } else {
                    print!("-"); 
                } 
            }
            print!("+\n");
        } else {
            for cell in &row.cells {
                  if cell.borrow_mut().walls.contains(&Wall::Top) {
                     
                    for i in 0..(Self::CELLWIDTHCHAR) {
                        if i % Self::CELLWIDTHCHAR == 0 {
                            print!("+");
                   
                        } else {
                            print!("-"); 
                        }    
                    }
                  } else {
                       for i in 0..(Self::CELLWIDTHCHAR) {
                            print!(" "); 
                       } 

                  } 
            }
            print!("+\n");
        }
    }

    pub fn print_vertical(&self,row: &Row) {
        for j in 0..Self::CELLHEIGHTCHAR {
            for i in 0..(self.width)*Self::CELLWIDTHCHAR + 1 {
                if i % Self::CELLWIDTHCHAR == 0 {
                    let ri = i /  Self::CELLWIDTHCHAR;
                    if ri !=self.width {
                        let c = &row.cells[ri];
                        if c.borrow_mut().walls.contains(&Wall::Left){
                            print!("|"); 
                        } else {
                            print!(" "); 
                        }
                    } else {
                         print!("|"); 
                    }  
                    
                } else {
                    if (i % (Self::CELLWIDTHCHAR/2) == 0) && (j == Self::CELLHEIGHTCHAR/2) {
                        let ri = i /  Self::CELLWIDTHCHAR;
                        let c = &row.cells[ri];
                        //print!("{}",c.borrow_mut().set_id); 
                        print!(" "); 
                    } else {
                        print!(" "); 
                    }
                    
                   
                } 
            
            }
            print!("\n"); 
        }
    }

    fn print(&self) {
        for (i,r) in self.rows.iter().enumerate() {
            let mut is_last= false;
            if i == self.rows.len() -1 
                 { is_last = true; } 
            
            self.print_horizontal(&r,false);
            self.print_vertical(&r);
            if is_last 
                { self.print_horizontal(&r,true); }
        }
    }


    
}

fn main() {
     let mut maze = Maze::new(8,12);    
     maze.construct(); 
     //println!("{:#?}", maze);  
     maze.print();
     
}
