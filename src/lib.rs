use std::str;

use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;

const TILE_SIZE: f64 = 64.0;
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
    riverew_end: Option<String>,
    riverew_roadns: Option<String>,
    riversw: Option<String>,
    riversw_roadne: Option<String>,
    river_start: Option<String>,
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
    let canvas_width = window.inner_width().unwrap().as_f64().unwrap() as u32;
    let canvas_height = window.inner_height().unwrap().as_f64().unwrap() as u32;
    canvas.set_width(canvas_width);
    canvas.set_height(canvas_height);

    let loaded_art =
        load_base_game_tiles(&tileart.base.expect("msg: Base game tile art is missing"));
    let loaded_art_len = loaded_art.len();
    wait_for_images(&loaded_art).await;
    log::info!("Finished loading tile art");

    let mut map = Map::new(
        loaded_art,
        1 + (canvas_width as f64 / TILE_SIZE) as u32,
        1 + (canvas_height as f64 / TILE_SIZE) as u32,
    );
    log::info!("Map created with size: {}x{}", map.size_x(), map.size_y());

    for y in 0..map.size_y() {
        for x in 0..map.size_x() {
            map.tiles[y as usize][x as usize] = Some(PlacedTile {
                tile_spec: 0,
                roation: 0,
            })
        }
    }

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    draw_map(&context, &map);
}

fn draw_map(context: &web_sys::CanvasRenderingContext2d, map: &Map) {
    for y in 0..map.size_y() {
        let pos_y = y as f64 * TILE_SIZE;
        for x in 0..map.size_x() {
            if let Some(tile) = &map.tiles[y as usize][x as usize] {
                let tile_spec = &map.specs[tile.tile_spec as usize];
                let pos_x = x as f64 * TILE_SIZE;
                draw_tile(context, tile_spec, pos_x, pos_y, tile.roation);
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
    roation: u8,
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

    // fn place_tile(&mut self, tile_spec: u8, rotation: u8, x: usize, y: usize) {
    //     if x < self.tiles.len() && y < self.tiles[x].len() {
    //         self.tiles[x].push(PlacedTile { tile_spec, roation: rotation });
    //     }
    // }
}

enum Feature {
    None,
    City,
    Road,
    River,
    Cloister,
}

struct TileSpec {
    cloister: bool,
    sheild: bool,
    edge_features: [Feature; 4],
    art: HtmlImageElement,
    count: i32,
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
