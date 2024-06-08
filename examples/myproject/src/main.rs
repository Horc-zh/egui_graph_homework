use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui::Pos2;
use egui::SidePanel;
use egui_graphs::SettingsNavigation;
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsInteraction, SettingsStyle,
};
use petgraph::stable_graph::{DefaultIx, EdgeIndex, NodeIndex, StableGraph};
use petgraph::Directed;
use reqwest::blocking::Client;
use scraper::node;
use scraper::{Html, Selector};
use std::env;
use std::error::Error;
use std::string;
use webbrowser;
pub struct BasicApp {
    g: Graph<(), (), Directed, DefaultIx>,
    init_called: bool,
    source_url: String,
    links: Vec<String>,

    selected_node: Option<NodeIndex>,
    selected_edge: Option<EdgeIndex>,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self {
            g: Graph::from(&g),
            init_called: false,
            source_url: String::default(),
            links: Vec::new(),
            selected_node: Option::default(),
            selected_edge: Option::default(),
        }
    }

    fn get_urls(&mut self) -> Result<(), Box<dyn Error>> {
        self.source_url.clear();
        self.links.clear();
        let args: Vec<String> = env::args().collect();

        if args.len() != 2 {
            eprintln!("Usage: {} <URL>", args[0]);
            panic!("there is no url entered!");
        }

        let url = &args[1];
        self.source_url = url.to_string();
        let client = Client::new();

        let res = client.get(url).send()?.text()?;

        let document = Html::parse_document(&res);

        let selector = Selector::parse("a").unwrap();

        for element in document.select(&selector) {
            if let Some(link) = element.value().attr("href") {
                if link.starts_with("http://") || link.starts_with("https://") {
                    self.links.push(link.to_string());
                }
            }
        }

        Ok(())
    }

    fn init(&mut self) {
        if !self.init_called {
            let _ = self.get_urls();

            let start = self.g.add_node_with_label_and_location(
                (),
                self.source_url.clone(),
                Pos2::new(200.0, 200.0),
            );

            let num_links = self.links.len();
            let radius = 150.0;
            let angle_step = 2.0 * 3.1415926 / num_links as f32;

            for (i, link) in self.links.iter().enumerate() {
                let angle = i as f32 * angle_step;
                let location =
                    Pos2::new(200.0 + radius * angle.cos(), 200.0 + radius * angle.sin());
                let end = self
                    .g
                    .add_node_with_label_and_location((), link.to_string(), location);
                self.g.add_edge(start, end, ());
            }
            self.init_called = true;
        }
    }

    fn read_data(&mut self) {
        if !self.g.selected_nodes().is_empty() {
            let idx = self.g.selected_nodes().first().unwrap();
            self.selected_node = Some(*idx);
            self.selected_edge = None;
        }
        if !self.g.selected_edges().is_empty() {
            let idx = self.g.selected_edges().first().unwrap();
            self.selected_edge = Some(*idx);
            self.selected_node = None;
        }
    }

    fn get_label(&self) -> Option<String> {
        if let Some(idx) = self.g.selected_nodes().first() {
            if let Some(node) = self.g.node(*idx) {
                Some(node.label())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.init();
        self.read_data();

        SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("click here to open or link an url");
            if ui.button("open").clicked() {
                if let Some(idx) = self.g.selected_nodes().first() {
                    if let Some(node) = self.g.node(*idx) {
                        if let Err(e) = webbrowser::open(&node.label()) {
                            eprintln!("Failed to open the browser: {}", e);
                        }
                    } else {
                        eprintln!("No node found with the given index.");
                    }
                } else {
                    eprintln!("No selected node.");
                }
            }

            //     if ui.button("link").clicked() {
            //         if let Some(url) = self.get_label() {
            //             println!("{url}");
            //             let _ = self.get_urls();
            //             println!("{:?}", self.links);
            //         }
            //         if let Some(rstart) = self.g.selected_nodes().first() {
            //             let start = *rstart;
            //             let radius = 150.0;
            //             let num_links = self.links.len();
            //             let angle_step = 2.0 * 3.1415926 / num_links as f32;

            //             for (i, link) in self.links.iter().enumerate() {
            //                 let angle = i as f32 * angle_step;
            //                 let location =
            //                     Pos2::new(200.0 + radius * angle.cos(), 200.0 + radius * angle.sin());
            //                 let end =
            //                     self.g
            //                         .add_node_with_label_and_location((), link.to_string(), location);
            //                 self.g.add_edge(start, end, ());
            //             }
            //         }
            //     }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let widget =
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.g)
                    .with_interactions(
                        &SettingsInteraction::default()
                            .with_node_selection_enabled(true)
                            .with_edge_selection_enabled(true)
                            .with_dragging_enabled(true),
                    )
                    .with_styles(&SettingsStyle::default().with_labels_always(true))
                    .with_navigations(
                        &SettingsNavigation::default().with_zoom_and_pan_enabled(true),
                    );
            ui.add(widget);
        });
    }
}

fn generate_graph() -> StableGraph<(), ()> {
    StableGraph::new()
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_demo",
        native_options,
        Box::new(|cc| Box::new(BasicApp::new(cc))),
    )
    .unwrap();
}
