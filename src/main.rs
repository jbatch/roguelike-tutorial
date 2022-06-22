use rltk::{GameState, Point, Rltk, RGB};
use specs::prelude::*;

mod player;
pub use player::*;
mod components;
pub use components::*;
mod map;
pub use map::*;
mod rect;
pub use rect::*;
mod visability_system;
pub use visability_system::*;
mod monster_ai_system;
pub use monster_ai_system::*;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

pub struct State {
    pub ecs: World,
    pub runstate: RunState,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisabilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (_player, viewshed) in (&mut players, &mut viewsheds).join() {
        let mut y = 0;
        let mut x = 0;

        for (idx, tile) in map.tiles.iter().enumerate() {
            // Render a tile depending on the tile type
            if map.revealed_tiles[idx] {
                let glyph;
                let mut fg;
                match tile {
                    TileType::Floor => {
                        glyph = rltk::to_cp437('.');
                        fg = RGB::from_f32(0.0, 0.5, 0.5);
                    }
                    TileType::Wall => {
                        glyph = rltk::to_cp437('#');
                        fg = RGB::from_f32(0., 1.0, 0.);
                    }
                }
                if !map.visible_tiles[idx] {
                    fg = fg.to_greyscale();
                }
                ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
            }

            // Move the coordinates
            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running,
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
        }
        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", &name, i),
            })
            .build();
    }
    gs.ecs.insert(map);
    gs.ecs.insert(Point {
        x: player_x,
        y: player_y,
    });
    gs.ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .build();

    rltk::main_loop(context, gs)
}
