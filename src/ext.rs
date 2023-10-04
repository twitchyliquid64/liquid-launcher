use chumsky::prelude::*;

pub trait ImmediateExtra {
    fn ui(&mut self, input: &String, ctx: &egui::Context, ui: &mut egui::Ui) -> bool;
}

#[derive(Debug, Eq, PartialEq)]
enum ParsedNumber {
    Decimal(u64),
}

impl ParsedNumber {
    fn decimal_str(&self) -> String {
        match self {
            ParsedNumber::Decimal(i) => i.to_string(),
        }
    }

    fn hex_str(&self) -> String {
        match self {
            ParsedNumber::Decimal(i) => (format!("{:#01x}", i)).to_string(),
        }
    }
    fn oct_str(&self) -> String {
        match self {
            ParsedNumber::Decimal(i) => (format!("{:#01o}", i)).to_string(),
        }
    }
    fn bin_str(&self) -> String {
        match self {
            ParsedNumber::Decimal(i) => (format!("{:#01b}", i)).to_string(),
        }
    }
}

#[derive(Default, Debug)]
pub struct NumFormatExtra {}

impl NumFormatExtra {
    fn parse(&self, input: &String) -> Option<ParsedNumber> {
        use chumsky::Parser;
        match parse_number().parse(input).into_result() {
            Ok(p) => Some(p),
            Err(_e) => None,
        }
    }
}

impl ImmediateExtra for NumFormatExtra {
    fn ui(&mut self, input: &String, ctx: &egui::Context, ui: &mut egui::Ui) -> bool {
        match self.parse(input) {
            Some(n) => {
                //ui.allocate_space(egui::Vec2::new(2., 0.));

                ui.columns(2, |columns| {
                    columns[0].allocate_space(egui::Vec2::new(2., 0.));
                    columns[0].heading("Integer representations");

                    columns[1].push_id("numberz", |ui| {
                        use egui_extras::{Column, TableBuilder};

                        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                        let mut table = TableBuilder::new(ui)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::initial(100.0).at_least(40.0))
                            .column(Column::initial(100.0).at_least(40.0))
                            .column(Column::initial(100.0).at_least(40.0))
                            .column(Column::remainder())
                            .auto_shrink([true, false]);

                        table
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Decimal");
                                });
                                header.col(|ui| {
                                    ui.strong("Hex");
                                });
                                header.col(|ui| {
                                    ui.strong("Oct");
                                });
                                header.col(|ui| {
                                    ui.strong("Binary");
                                });
                            })
                            .body(|mut body| {
                                body.row(text_height, |mut row| {
                                    row.col(|ui| {
                                        ui.label(n.decimal_str());
                                    });
                                    row.col(|ui| {
                                        ui.label(n.hex_str());
                                    });
                                    row.col(|ui| {
                                        ui.label(n.oct_str());
                                    });
                                    row.col(|ui| {
                                        ui.style_mut().wrap = Some(false);
                                        ui.label(n.bin_str());
                                    });
                                })
                            });
                    });
                });

                true
            }
            None => false,
        }
    }
}

fn parse_number<'a>() -> impl Parser<'a, &'a str, ParsedNumber> {
    //let ident = text::ident().padded();

    let int = text::int(10).map(|s: &str| ParsedNumber::Decimal(s.parse().unwrap_or(u64::MAX)));

    int
}
