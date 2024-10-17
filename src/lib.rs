use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use gloo::events::EventListener;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Element, HtmlElement};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // Better error messages in debug mode.
    console_error_panic_hook::set_once();

    start_rendering_loop();

    Ok(())
}

fn start_rendering_loop() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let card = document.get_element_by_id("card").unwrap();

    let is_mouse_down = Rc::new(RefCell::new(false));
    // Appease the borrow checker
    let is_mouse_down_cloned = Rc::clone(&is_mouse_down);
    let is_mouse_down_cloned_2 = Rc::clone(&is_mouse_down);

    let _on_mouse_down = EventListener::new(&document, "pointerdown", move |_event| {
        *is_mouse_down.borrow_mut() = true;
    })
    .forget(); // Listen forever

    let _on_mouse_up = EventListener::new(&document, "pointerup", move |_event| {
        *is_mouse_down_cloned.borrow_mut() = false;
    })
    .forget();

    let mask: Mask = [false; AREA];
    let mask = Rc::new(RefCell::new(mask));
    let mask_cloned = Rc::clone(&mask);

    let _on_mouse_move = EventListener::new(&card, "pointermove", move |e| {
        let event = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
        let rect = event
            .target()
            .expect("mouse event doesn't have a target")
            .dyn_into::<HtmlElement>()
            .expect("event target should be of type HtmlElement")
            .get_bounding_client_rect();

        let x = (event.client_x() as f64) - rect.left();
        let y = (event.client_y() as f64) - rect.top();

        let mask_index: usize = transform_mouse_coordinates_to_mask_index(x, y);

        if *is_mouse_down_cloned_2.borrow() && mask_index < AREA {
            let mut borrowed_mask = *mask.borrow_mut();
            // "Scratch" the card
            borrowed_mask[mask_index] = true;
            *mask.borrow_mut() = borrowed_mask;
        }
    })
    .forget();

    // Rendering loop inspired by
    // https://github.com/rustwasm/wasm-bindgen/blob/5e98a17da61dcc59b16b6fcb52f9d96517518025/examples/request-animation-frame/src/lib.rs
    let rendering_callback_cell_one = Rc::new(RefCell::new(None));
    let rendering_callback_cell_two = rendering_callback_cell_one.clone();

    *rendering_callback_cell_two.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        render(&card, mask_cloned.borrow());

        request_animation_frame(rendering_callback_cell_one.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(rendering_callback_cell_two.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    let window = web_sys::window().unwrap();

    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Unable to `requestAnimationFrame`");
}

const GLYPH_WIDTH: f64 = 10.0;
const GLYPH_HEIGHT: f64 = 15.0; 
const LINE_SPACING: f64 = 13.0;


const WIDTH_COMPENSATION: f64 = GLYPH_WIDTH / 115.0;
const HEIGHT_COMPENSATION: f64 = (GLYPH_HEIGHT + LINE_SPACING) / 868.0;

/**
* The text characters are stored in a 1d array, while the mouse is in 2d.
* These magic numbers let us transform the mouse coordinates to the mask coordinate space.
*/
fn transform_mouse_coordinates_to_mask_index(x: f64, y: f64) -> usize {
    let x_index = (x * WIDTH_COMPENSATION) as usize - 1;
    let y_index = (y * HEIGHT_COMPENSATION) as usize;
    y_index * WIDTH + x_index
}

const WIDTH: usize = 40;
const HEIGHT: usize = 6;
const AREA: usize = WIDTH * HEIGHT;

type Mask = [bool; AREA];
type Characters = [char; AREA];

fn render(target: &Element, mask: Ref<Mask>) {
    let characters = get_characters(*mask);
    let string: String = characters.iter().collect();
    target.set_text_content(Some(&string));
}

const MESSAGE: [&str; HEIGHT] = [
    "",
    "    Dear people at Zoo, ",
    "    would you like to ",
    "    build the future ",
    "    of hardware design together? ",
    "", 
];

const PADDING: usize = 2;

fn get_characters(mask: Mask) -> Characters {
    let mut characters: Characters = ['x'; AREA];

    for index in 0..AREA {
        let x_index = index % WIDTH;

        if x_index == 0 {
            characters[index] = '\n';
        } else {
            let y_index = index / WIDTH;
            let sentence = MESSAGE[y_index];
            let is_message_index = x_index > PADDING && y_index != 0 && y_index != sentence.len();

            let pattern_index = x_index % 3;
            let top_layer = match pattern_index {
                _ => '0',
            };
            let bottom_layer = match pattern_index - 1 {
                0 => 'Z',
                _ => 'O',
            };

            characters[index] = if mask[index] {
                if is_message_index {
                    let message_row = sentence.as_bytes();
                    let row_has_character = x_index < message_row.len();
                    match row_has_character {
                        true => message_row[x_index] as char,
                        false => top_layer,
                    }
                } else {
                    top_layer
                }
            } else {
                bottom_layer
            }
        }
    }

    characters
}
