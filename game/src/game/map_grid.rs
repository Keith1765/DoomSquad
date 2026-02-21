use std::{collections::HashMap, thread::current};
use crate::game::{entities::{self, Entity}, interactables::Interactable, map::Point};

//grid in each cell of MapGrid
# [derive(PartialEq, Eq, Hash,Clone)]
pub struct Cell {
    x: i32,
    y: i32,
}

# [derive(Clone)]
pub struct MapGrid {
    pub grid: HashMap <Cell, Vec<usize>>,
    pub cell_size: f64, //size of one cell in world units
}

impl MapGrid{

    pub fn new(cell_size: f64) -> Self {
        Self{
            grid: HashMap::new(),
            cell_size,
        }
    }

    pub fn getCell(&self, position: Point) -> Cell {
        Cell{
        x: (position.x / self.cell_size) as i32,
        y: (position.y / self.cell_size) as i32,
        }
    } 

    //implemented for entities
    pub fn update(&mut self, entities: &Vec<Entity>) {
        //clear last frame
        self.grid.clear();

        for (id, entity) in entities.iter().enumerate() {
            let cell = self.getCell(entity.mover.position);
            self.grid.entry(cell).or_default().push(id);
        }
    }

    //neighbours by 1 so 3x3
    pub fn get_neighbours (&mut self, position: Point) -> Vec<usize>{
        let mut entities = Vec::new();
        let current_cell = self.getCell(position);

        for dx in -1 ..= 1 {
            for dy in -1 ..= 1 {
                let cell = Cell{ 
                    x: current_cell.x + dx ,
                    y:  current_cell.y + dy};

                if let Some(neighbour) = self.grid.get(&cell){
                    entities.extend(neighbour);
                }
            }
        }

        entities
    }
    //implemetation for interactables 
    pub fn update_interactables(&mut self, interactables: &Vec<Interactable>) {
        self.grid.clear();

        for (id, interactable) in interactables.iter().enumerate() {
            let cell = self.getCell(interactable.mover.position);
            self.grid.entry(cell).or_default().push(id);
        }
    }

    pub fn get_interactable_neighbours(&self, position: Point) -> Vec<usize> {
    let mut interactables = Vec::new();
    let current_cell = self.getCell(position);

    for dx in -1..=1 {
        for dy in -1..=1 {
            let cell = Cell {
                x: current_cell.x + dx,
                y: current_cell.y + dy,
            };

            if let Some(neighbour) = self.grid.get(&cell) {
                interactables.extend(neighbour);
            }
        }
    }

    interactables
}

}

