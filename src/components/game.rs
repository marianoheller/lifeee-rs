use crate::components::board::Board;
use crate::components::pattern_selector::PatternSelector;
use crate::lexicon::Term;
use crate::life::*;
use crate::Settings;
use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use std::collections::VecDeque;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct Game {
  cells: CellSet,
  previous_gens: Vec<CellSet>,
  tick: u32,
  interval: Option<Interval>,
  speed: u8,
  adjust_offset: Option<(usize, usize)>,
  offset: (f64, f64),
  zoom: f64,
  width: u32,
  height: u32,
  _resize_handle: EventListener,
}

pub enum Msg {
  NextTick,
  Play,
  Pause,
  ChangeSpeed(u8),
  ApplyPattern(Term),
  ChangeZoomAndOffset((Option<f64>, Option<(f64, f64)>)),
  Resize,
}

impl Game {
  fn settings(&self, ctx: &Context<Self>) -> Settings {
    ctx
      .link()
      .context::<Settings>(Callback::noop())
      .expect("settings context to be set")
      .0
  }

  fn start_interval(&mut self, ctx: &Context<Self>) {
    let link = ctx.link().clone();
    link.send_message(Msg::NextTick);
    let millis = (50_f64 - 500_f64) / 9_f64 * self.speed as f64 + 500_f64;
    let interval = Interval::new(millis as u32, move || link.send_message(Msg::NextTick));
    self.interval = Some(interval);
  }
}

impl Component for Game {
  type Message = Msg;
  type Properties = ();

  fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    let settings = self.settings(ctx);
    match msg {
      Msg::NextTick => {
        self.tick += 1;
        self.adjust_offset = None;

        self.previous_gens = {
          let mut previous_gens_deque: VecDeque<CellSet> = self
            .previous_gens
            .iter()
            .map(|cell_set| cell_set.clone())
            .collect();
          previous_gens_deque.push_front(self.cells.clone());
          if previous_gens_deque.len() > settings.num_previous {
            previous_gens_deque.pop_back();
          }
          previous_gens_deque
            .iter()
            .map(|cell_set| cell_set.clone())
            .collect()
        };

        self.cells = tick(&self.cells);

        true
      }
      Msg::Play => {
        self.start_interval(ctx);
        true
      }
      Msg::Pause => {
        self.interval = None;
        true
      }
      Msg::ChangeSpeed(speed) => {
        self.speed = speed;
        if self.interval.is_some() {
          self.start_interval(ctx);
        }
        true
      }
      Msg::ApplyPattern(term) => {
        self.cells = term
          .cells
          .iter()
          .fold(CellSet::new(), |cells, &cell| make_cell_alive(&cells, cell));
        self.tick = 0;
        self.previous_gens = vec![];
        self.adjust_offset = Some((term.width, term.height));
        true
      }
      Msg::ChangeZoomAndOffset((zoom, offset)) => {
        if let Some(zoom) = zoom {
          self.zoom = zoom;
        }
        if let Some(offset) = offset {
          self.offset = offset;
        }
        true
      }
      Msg::Resize => {
        let window = web_sys::window().unwrap();
        let (width, height) = (
          window.inner_width().unwrap().as_f64().unwrap() as u32,
          window.inner_height().unwrap().as_f64().unwrap() as u32,
        );
        self.width = width;
        self.height = height;
        true
      }
    }
  }

  fn create(ctx: &Context<Self>) -> Self {
    let window = web_sys::window().unwrap();
    let link = ctx.link().clone();
    let resize_handle = EventListener::new(&window, "resize", move |_: &Event| {
      link.send_message(Msg::Resize)
    });

    Self {
      cells: CellSet::new(),
      previous_gens: vec![] as Vec<CellSet>,
      tick: 0,
      interval: None,
      speed: 5,
      adjust_offset: None,
      offset: (0.0, 0.0),
      zoom: 1.0,
      width: 300,
      height: 200,
      _resize_handle: resize_handle,
    }
  }

  fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
    if _first_render {
      ctx.link().send_message(Msg::Resize);
    }
  }

  fn view(&self, ctx: &Context<Self>) -> yew::virtual_dom::VNode {
    let running = self.interval.is_some();

    let on_change_speed = ctx.link().callback(|event: Event| {
      let input = event
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap();
      let speed: u8 = input.value().parse().unwrap();
      Msg::ChangeSpeed(speed)
    });

    html! {
      <>
        <Board
          cells={self.cells.clone()}
          previous_gens={self.previous_gens.clone()}
          offset={self.offset}
          zoom={self.zoom}
          change_zoom_and_offset={ctx.link().callback(move |(zoom, offset)| Msg::ChangeZoomAndOffset((zoom, offset)))}
          width={self.width}
          height={self.height}
        />
        <div style="background: white; position: absolute; bottom: 10px; left: 10px">
          <button disabled={running} onclick={ctx.link().callback(|_| Msg::NextTick)}>{"Tick"}</button>
          <button onclick={
            if running {
              ctx.link().callback(|_| Msg::Pause)
            } else {
              ctx.link().callback(|_| Msg::Play)
            }
          }>{{if running { "Pause" } else { "Play" }}}</button>
          <label>
            {"Speed:"}
            <input
              type="range" min="1" max="10"
              onchange={on_change_speed}
            />
          </label>
          <PatternSelector on_apply_pattern={ctx.link().callback(|term| Msg::ApplyPattern(term))} />
          <p>{"Generation #"}{self.tick}</p>
        </div>
      </>
    }
  }
}
