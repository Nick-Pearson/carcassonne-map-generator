use std::str;

use log::info;
use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;

const TILE_SIZE: f64 = 128.0;
const HALF_TILE_SIZE: f64 = TILE_SIZE / 2.0;

#[derive(serde::Deserialize)]
pub struct BaseGameTileArt {
    pub cityew: Option<String>,
    pub cityew_shield: Option<String>,
    pub cityn: Option<String>,
    pub citynesw_shield: Option<String>,
    pub citynew: Option<String>,
    pub citynew_roads: Option<String>,
    pub citynew_roads_shield: Option<String>,
    pub citynew_shield: Option<String>,
    pub citynw: Option<String>,
    pub citynw_roades: Option<String>,
    pub citynw_roades_shield: Option<String>,
    pub citynw_shield: Option<String>,
    pub cityn_citys: Option<String>,
    pub cityn_cityw: Option<String>,
    pub cityn_roades: Option<String>,
    pub cityn_roadesw: Option<String>,
    pub cityn_roadew: Option<String>,
    pub cityn_roadsw: Option<String>,
    pub cloister: Option<String>,
    pub cloister_roads: Option<String>,
    pub roadesw: Option<String>,
    pub roadnesw: Option<String>,
    pub roadns: Option<String>,
    pub roadsw: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct RiverTileArt {
    cloister_riverew_roads: Option<String>,
    riveres_citynw: Option<String>,
    riverew: Option<String>,
    riverew_cityn_citys: Option<String>,
    riverew_cityn_roads: Option<String>,
    riverew_roadns: Option<String>,
    riversw: Option<String>,
    riversw_roadne: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct TileArt {
    pub base: Option<BaseGameTileArt>,
    pub river: Option<RiverTileArt>,
}

#[wasm_bindgen]
pub fn init() {
    wasm_logger::init(wasm_logger::Config::default());
}

#[wasm_bindgen]
pub async fn render_map(tileart_js: JsValue) {
    log::info!("Rendering map...");
    let tileart: TileArt =
        serde_wasm_bindgen::from_value(tileart_js).expect("failed to deserialize TileArt");
    log::info!("Loading tile art...");

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    // let canvas_width = window.inner_width().unwrap().as_f64().unwrap() as u32;
    // let canvas_height = window.inner_height().unwrap().as_f64().unwrap() as u32;
    let canvas_width = 4961;
    let canvas_height = 7016;
    canvas.set_width(canvas_width);
    canvas.set_height(canvas_height);

    let base_game_art =
        load_base_game_tiles(&tileart.base.expect("msg: Base game tile art is missing"));
    let base_game_art_len = base_game_art.len();
    let river_art = load_river_game_tiles(&tileart.river.expect("msg: River tile art is missing"));
    let river_art_len = river_art.len();
    let all_art = [base_game_art, river_art].concat();
    wait_for_images(&all_art).await;
    log::info!("Finished loading tile art");

    let mut map = Map::new(
        all_art,
        1 + (canvas_width as f64 / TILE_SIZE) as u32,
        1 + (canvas_height as f64 / TILE_SIZE) as u32,
    );
    log::info!("Map created with size: {}x{}", map.size_x(), map.size_y());

    for _ in 0..1000 {
        let result = place_river_tiles(
            &mut map,
            base_game_art_len..base_game_art_len + river_art_len,
        );
        if result {
            info!("River tiles placed successfully");
            break;
        } else {
            info!("Failed to place river tiles, retrying...");
            map.clear_tiles();
        }
    }

    place_remaining_tiles(&mut map, 0..base_game_art_len);

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    draw_map(&context, &map);
}

fn place_river_tiles(map: &mut Map, river_art_range: std::ops::Range<usize>) -> bool {
    let start_x = (js_sys::Math::random() * (map.size_x() as f64)) as usize;
    let start_y = (js_sys::Math::random() * (map.size_y() as f64)) as usize;

    let draw_deck = build_draw_deck(map, river_art_range);
    let mut remaining: Vec<(usize, usize)> = Vec::new();

    place_river_tile(map, &mut remaining, start_x, start_y, draw_deck[0], 0);

    while let Some((x, y)) = remaining.pop() {
        let mut placed = false;
        for _ in 0..100 {
            let rotation = js_sys::Math::floor(js_sys::Math::random() * 4.0) as u8;
            let deck_idx =
                js_sys::Math::floor(js_sys::Math::random() * draw_deck.len() as f64) as usize;
            let selected_card = draw_deck[deck_idx];

            if map.can_be_placed(selected_card, x, y, rotation) {
                place_river_tile(map, &mut remaining, x, y, selected_card, rotation);
                placed = true;
                break;
            }
        }

        if !placed {
            // If we couldn't place any tile here, we stop trying
            return false;
        }
    }
    true
}

fn place_river_tile(
    map: &mut Map,
    remaining: &mut Vec<(usize, usize)>,
    x: usize,
    y: usize,
    tile: u8,
    rotation: u8,
) {
    let tile_spec = &map.specs[tile as usize];

    map.tiles[y][x] = Some(PlacedTile {
        tile_spec: tile,
        rotation,
    });

    for i in 0..4 {
        let edge_feature = tile_spec.edge_features[(i + rotation as usize) % 4];
        if edge_feature == Feature::River {
            let (dx, dy) = match i {
                0 => (0, -1), // North
                1 => (1, 0),  // East
                2 => (0, 1),  // South
                3 => (-1, 0), // West
                _ => unreachable!(),
            };
            let new_x = (x as i32) + dx;
            let new_y = (y as i32) + dy;

            if map.is_valid_position(new_x, new_y) && map.has_no_tile(new_x, new_y) {
                remaining.push((new_x as usize, new_y as usize));
            }
        }
    }
}

fn place_remaining_tiles(map: &mut Map, range: std::ops::Range<usize>) {
    let draw_deck = build_draw_deck(map, range);
    let mut remaining: Vec<(usize, usize)> = Vec::new();

    for y in 0..map.size_y() {
        for x in 0..map.size_x() {
            if map.has_no_tile(x as i32, y as i32) {
                remaining.push((x as usize, y as usize));
            }
        }
    }

    info!("Remaining tiles to place: {}", remaining.len());

    while let Some((x, y)) = remaining.pop() {
        for _ in 0..15000 {
            let deck_idx =
                js_sys::Math::floor(js_sys::Math::random() * draw_deck.len() as f64) as usize;
            let selected_card = draw_deck[deck_idx];
            let selected_tile_spec = &map.specs[selected_card as usize];
            let rotation = match selected_tile_spec.can_be_rotated() {
                true => js_sys::Math::floor(js_sys::Math::random() * 4.0) as u8,
                false => 0, // If the tile cannot be rotated, we use rotation 0
            };

            if map.can_be_placed(selected_card, x, y, rotation) {
                map.tiles[y][x] = Some(PlacedTile {
                    tile_spec: selected_card,
                    rotation,
                });
                break;
            }
        }
    }
}

fn build_draw_deck(map: &mut Map, range: std::ops::Range<usize>) -> Vec<u8> {
    let mut deck = Vec::new();
    for i in range {
        let tile_spec = &map.specs[i];
        for _ in 0..tile_spec.count {
            deck.push(i as u8);
        }
    }
    deck
}

fn draw_map(context: &web_sys::CanvasRenderingContext2d, map: &Map) {
    for y in 0..map.size_y() {
        let pos_y = y as f64 * TILE_SIZE;
        for x in 0..map.size_x() {
            if let Some(tile) = &map.tiles[y as usize][x as usize] {
                let tile_spec = &map.specs[tile.tile_spec as usize];
                let pos_x = x as f64 * TILE_SIZE;
                draw_tile(context, tile_spec, pos_x, pos_y, tile.rotation);
            }
        }
    }
}

fn draw_tile(
    context: &web_sys::CanvasRenderingContext2d,
    tile: &TileSpec,
    x: f64,
    y: f64,
    rotation: u8,
) {
    context.save();
    context
        .translate(x + HALF_TILE_SIZE, y + HALF_TILE_SIZE)
        .unwrap();
    for _ in 0..rotation {
        context.rotate(-std::f64::consts::FRAC_PI_2).unwrap();
    }
    context
        .draw_image_with_html_image_element_and_dw_and_dh(
            &tile.art,
            -HALF_TILE_SIZE,
            -HALF_TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
        )
        .unwrap();
    context.restore();
}

struct PlacedTile {
    tile_spec: u8,
    rotation: u8,
}

struct Map {
    size_x: u32,
    size_y: u32,
    specs: Vec<TileSpec>,
    tiles: Vec<Vec<Option<PlacedTile>>>,
}

impl Map {
    fn new(specs: Vec<TileSpec>, size_x: u32, size_y: u32) -> Self {
        let mut tiles = Vec::new();
        for _ in 0..size_y {
            let mut row = Vec::new();
            for _ in 0..size_x {
                row.push(None);
            }
            tiles.push(row);
        }
        Map {
            size_x,
            size_y,
            specs,
            tiles,
        }
    }

    fn size_x(&self) -> u32 {
        self.size_x
    }

    fn size_y(&self) -> u32 {
        self.size_y
    }

    fn is_valid_position(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.size_x && (y as u32) < self.size_y
    }

    fn has_no_tile(&self, x: i32, y: i32) -> bool {
        if !self.is_valid_position(x, y) {
            return false;
        }
        self.tiles[y as usize][x as usize].is_none()
    }

    fn can_be_placed(&self, tile: u8, x: usize, y: usize, rotation: u8) -> bool {
        let tile_spec = &self.specs[tile as usize];
        for i in 0..4 {
            let edge_feature: Feature = tile_spec.edge_features[(i + rotation as usize) % 4];
            let (dx, dy) = match i {
                0 => (0, -1), // North
                1 => (1, 0),  // East
                2 => (0, 1),  // South
                3 => (-1, 0), // West
                _ => unreachable!(),
            };

            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            if self.is_valid_position(new_x, new_y) {
                if let Some(other_tile) = &self.tiles[new_y as usize][new_x as usize] {
                    let other_tile_spec = &self.specs[other_tile.tile_spec as usize];
                    let other_edge_feature =
                        other_tile_spec.edge_features[(2 + i + other_tile.rotation as usize) % 4];
                    if other_edge_feature != edge_feature {
                        // If the edge features do not match, we cannot place the tile
                        return false;
                    }
                }
            }
        }
        true
    }

    fn clear_tiles(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                *tile = None;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Feature {
    None,
    City,
    Road,
    River,
}

#[derive(Clone, Debug)]
struct TileSpec {
    cloister: bool,
    sheild: bool,
    edge_features: [Feature; 4],
    art: HtmlImageElement,
    count: i32,
}

impl TileSpec {
    fn can_be_rotated(&self) -> bool {
        !self
            .edge_features
            .iter()
            .all(|&f| f == self.edge_features[0])
    }
}

fn load_image(url: &str) -> Result<HtmlImageElement, JsValue> {
    let img = HtmlImageElement::new()?;
    img.set_src(url);
    Ok::<_, JsValue>(img)
}

fn load_base_game_tiles(art: &BaseGameTileArt) -> Vec<TileSpec> {
    let mut tiles = Vec::new();

    if let Some(cityew) = &art.cityew {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::None, Feature::City, Feature::None, Feature::City],
            art: load_image(cityew).unwrap(),
            count: 1,
        });
    }

    if let Some(cityew_shield) = &art.cityew_shield {
        tiles.push(TileSpec {
            cloister: false,
            sheild: true,
            edge_features: [Feature::None, Feature::City, Feature::None, Feature::City],
            art: load_image(cityew_shield).unwrap(),
            count: 2,
        });
    }

    if let Some(cityn) = &art.cityn {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::None, Feature::None, Feature::None],
            art: load_image(cityn).unwrap(),
            count: 5,
        });
    }
    if let Some(citynesw_shield) = &art.citynesw_shield {
        tiles.push(TileSpec {
            cloister: false,
            sheild: true,
            edge_features: [Feature::City, Feature::City, Feature::City, Feature::City],
            art: load_image(citynesw_shield).unwrap(),
            count: 1,
        });
    }
    if let Some(citynew) = &art.citynew {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::City, Feature::None, Feature::City],
            art: load_image(citynew).unwrap(),
            count: 3,
        });
    }
    if let Some(citynew_roads) = &art.citynew_roads {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::City, Feature::Road, Feature::City],
            art: load_image(citynew_roads).unwrap(),
            count: 1,
        });
    }
    if let Some(citynew_roads_shield) = &art.citynew_roads_shield {
        tiles.push(TileSpec {
            cloister: false,
            sheild: true,
            edge_features: [Feature::City, Feature::City, Feature::Road, Feature::City],
            art: load_image(citynew_roads_shield).unwrap(),
            count: 1,
        });
    }

    if let Some(citynew_shield) = &art.citynew_shield {
        tiles.push(TileSpec {
            cloister: false,
            sheild: true,
            edge_features: [Feature::City, Feature::City, Feature::None, Feature::City],
            art: load_image(citynew_shield).unwrap(),
            count: 1,
        });
    }

    if let Some(citynw) = &art.citynw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::None, Feature::None, Feature::City],
            art: load_image(citynw).unwrap(),
            count: 3,
        });
    }

    if let Some(citynw_roades) = &art.citynw_roades {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::Road, Feature::Road, Feature::City],
            art: load_image(citynw_roades).unwrap(),
            count: 3,
        });
    }

    if let Some(citynw_roades_shield) = &art.citynw_roades_shield {
        tiles.push(TileSpec {
            cloister: false,
            sheild: true,
            edge_features: [Feature::City, Feature::Road, Feature::Road, Feature::City],
            art: load_image(citynw_roades_shield).unwrap(),
            count: 2,
        });
    }

    if let Some(citynw_shield) = &art.citynw_shield {
        tiles.push(TileSpec {
            cloister: false,
            sheild: true,
            edge_features: [Feature::City, Feature::None, Feature::None, Feature::City],
            art: load_image(citynw_shield).unwrap(),
            count: 2,
        });
    }

    if let Some(cityn_citys) = &art.cityn_citys {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::None, Feature::City, Feature::None],
            art: load_image(cityn_citys).unwrap(),
            count: 3,
        });
    }

    if let Some(cityn_cityw) = &art.cityn_cityw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::None, Feature::None, Feature::City],
            art: load_image(cityn_cityw).unwrap(),
            count: 2,
        });
    }

    if let Some(cityn_roades) = &art.cityn_roades {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::Road, Feature::Road, Feature::None],
            art: load_image(cityn_roades).unwrap(),
            count: 3,
        });
    }

    if let Some(cityn_roadesw) = &art.cityn_roadesw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::Road, Feature::Road, Feature::Road],
            art: load_image(cityn_roadesw).unwrap(),
            count: 3,
        });
    }

    if let Some(cityn_roadew) = &art.cityn_roadew {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::Road, Feature::None, Feature::Road],
            art: load_image(cityn_roadew).unwrap(),
            count: 4,
        });
    }

    if let Some(cityn_roadsw) = &art.cityn_roadsw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::None, Feature::Road, Feature::Road],
            art: load_image(cityn_roadsw).unwrap(),
            count: 3,
        });
    }

    if let Some(cloister) = &art.cloister {
        tiles.push(TileSpec {
            cloister: true,
            sheild: false,
            edge_features: [Feature::None, Feature::None, Feature::None, Feature::None],
            art: load_image(cloister).unwrap(),
            count: 4,
        });
    }

    if let Some(cloister_roads) = &art.cloister_roads {
        tiles.push(TileSpec {
            cloister: true,
            sheild: false,
            edge_features: [Feature::None, Feature::None, Feature::Road, Feature::None],
            art: load_image(cloister_roads).unwrap(),
            count: 2,
        });
    }

    if let Some(roadesw) = &art.roadesw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::None, Feature::Road, Feature::Road, Feature::Road],
            art: load_image(roadesw).unwrap(),
            count: 4,
        });
    }

    if let Some(roadnesw) = &art.roadnesw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::Road, Feature::Road, Feature::Road, Feature::Road],
            art: load_image(roadnesw).unwrap(),
            count: 1,
        });
    }

    if let Some(roadns) = &art.roadns {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::Road, Feature::None, Feature::Road, Feature::None],
            art: load_image(roadns).unwrap(),
            count: 8,
        });
    }

    if let Some(roadsw) = &art.roadsw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::None, Feature::None, Feature::Road, Feature::Road],
            art: load_image(roadsw).unwrap(),
            count: 9,
        });
    }

    tiles
}

fn load_river_game_tiles(art: &RiverTileArt) -> Vec<TileSpec> {
    let mut tiles = Vec::new();

    if let Some(riverew) = &art.riverew {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::None, Feature::River, Feature::None, Feature::River],
            art: load_image(riverew).unwrap(),
            count: 1,
        });
    }
    if let Some(cloister_riverew_roads) = &art.cloister_riverew_roads {
        tiles.push(TileSpec {
            cloister: true,
            sheild: false,
            edge_features: [Feature::None, Feature::River, Feature::Road, Feature::River],
            art: load_image(cloister_riverew_roads).unwrap(),
            count: 1,
        });
    }
    if let Some(riveres_citynw) = &art.riveres_citynw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::River, Feature::River, Feature::City],
            art: load_image(riveres_citynw).unwrap(),
            count: 1,
        });
    }
    if let Some(riverew_cityn_citys) = &art.riverew_cityn_citys {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::River, Feature::City, Feature::River],
            art: load_image(riverew_cityn_citys).unwrap(),
            count: 1,
        });
    }

    if let Some(riverew_cityn_roads) = &art.riverew_cityn_roads {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::City, Feature::River, Feature::Road, Feature::River],
            art: load_image(riverew_cityn_roads).unwrap(),
            count: 1,
        });
    }

    if let Some(riverew_roadns) = &art.riverew_roadns {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::Road, Feature::River, Feature::Road, Feature::River],
            art: load_image(riverew_roadns).unwrap(),
            count: 1,
        });
    }

    if let Some(riversw) = &art.riversw {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::None, Feature::None, Feature::River, Feature::River],
            art: load_image(riversw).unwrap(),
            count: 1,
        });
    }

    if let Some(riversw_roadne) = &art.riversw_roadne {
        tiles.push(TileSpec {
            cloister: false,
            sheild: false,
            edge_features: [Feature::Road, Feature::Road, Feature::River, Feature::River],
            art: load_image(riversw_roadne).unwrap(),
            count: 1,
        });
    }

    tiles
}

/// Waits for all images in the provided vector to finish loading.
async fn wait_for_images(tiles: &[TileSpec]) {
    for tile in tiles {
        let img_promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let onload = Closure::once_into_js(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            tile.art.set_onload(Some(onload.unchecked_ref()));
        });

        if tile.art.complete() {
            continue; // Image already loaded
        }

        wasm_bindgen_futures::JsFuture::from(img_promise)
            .await
            .expect("Image failed to load");
    }
}
