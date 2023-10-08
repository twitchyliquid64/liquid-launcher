use chumsky::prelude::*;
use egui_extras::{Size, StripBuilder};

pub trait ImmediateExtra {
    fn ui(&mut self, input: &String, ctx: &egui::Context, ui: &mut egui::Ui) -> bool;
}

#[derive(Debug, Eq, PartialEq)]
enum ParsedNumber {
    Decimal(u64),
    Binary(u64),
    Hex(u64),
    Oct(u64),
}

impl ParsedNumber {
    fn decimal_str(&self) -> String {
        match self {
            ParsedNumber::Decimal(i)
            | ParsedNumber::Binary(i)
            | ParsedNumber::Hex(i)
            | ParsedNumber::Oct(i) => i.to_string(),
        }
    }

    fn hex_str(&self) -> String {
        match self {
            ParsedNumber::Decimal(i)
            | ParsedNumber::Binary(i)
            | ParsedNumber::Hex(i)
            | ParsedNumber::Oct(i) => (format!("{:#01x}", i)).to_string(),
        }
    }
    fn oct_str(&self) -> String {
        match self {
            ParsedNumber::Decimal(i)
            | ParsedNumber::Binary(i)
            | ParsedNumber::Hex(i)
            | ParsedNumber::Oct(i) => (format!("{:#01o}", i)).to_string(),
        }
    }
    fn bin_str(&self) -> String {
        match self {
            ParsedNumber::Decimal(i)
            | ParsedNumber::Binary(i)
            | ParsedNumber::Hex(i)
            | ParsedNumber::Oct(i) => (format!("{:#01b}", i)).to_string(),
        }
    }
}

#[derive(Default, Debug)]
pub struct NumFormatExtra {}

impl NumFormatExtra {
    fn parse(&self, input: &String) -> Option<ParsedNumber> {
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

                StripBuilder::new(ui)
                    .size(Size::relative(0.3)) // left cell
                    .size(Size::remainder().at_least(200.0)) // right cell
                    .horizontal(|mut strip| {
                        strip.cell(|ui| {
                            ui.allocate_space(egui::Vec2::new(2., 0.));
                            ui.heading("Integer representations");
                        });
                        strip.cell(|ui| {
                            ui.push_id("numberz", |ui| {
                                use egui_extras::{Column, TableBuilder};

                                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                                let table = TableBuilder::new(ui)
                                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                    .column(Column::initial(100.0).at_least(40.0))
                                    .column(Column::initial(40.0))
                                    .column(Column::remainder())
                                    .auto_shrink([true, false]);

                                table.body(|mut body| {
                                    body.row(text_height, |mut row| {
                                        row.col(|ui| {
                                            ui.strong("Decimal");
                                        });
                                        row.col(|ui| {
                                            if ui.small_button("📋").clicked() {
                                                ctx.output_mut(|o| o.copied_text = n.decimal_str());
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(n.decimal_str());
                                        });
                                    });
                                    body.row(text_height, |mut row| {
                                        row.col(|ui| {
                                            ui.strong("Hex");
                                        });
                                        row.col(|ui| {
                                            if ui.small_button("📋").clicked() {
                                                ctx.output_mut(|o| o.copied_text = n.hex_str());
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(n.hex_str());
                                        });
                                    });
                                    body.row(text_height, |mut row| {
                                        row.col(|ui| {
                                            ui.strong("Oct");
                                        });
                                        row.col(|ui| {
                                            if ui.small_button("📋").clicked() {
                                                ctx.output_mut(|o| o.copied_text = n.oct_str());
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(n.oct_str());
                                        });
                                    });
                                    body.row(text_height, |mut row| {
                                        row.col(|ui| {
                                            ui.strong("Binary");
                                        });
                                        row.col(|ui| {
                                            if ui.small_button("📋").clicked() {
                                                ctx.output_mut(|o| o.copied_text = n.bin_str());
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.style_mut().wrap = Some(false);
                                            ui.label(n.bin_str());
                                        });
                                    });
                                });
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
    let bin = just("0b").then(text::int(2)).map(|(_, s): (&str, &str)| {
        ParsedNumber::Binary(u64::from_str_radix(s, 2).unwrap_or(u64::MAX))
    });
    let hex = just("0x").then(text::int(16)).map(|(_, s): (&str, &str)| {
        ParsedNumber::Hex(u64::from_str_radix(s, 16).unwrap_or(u64::MAX))
    });
    let oct = just("0o").then(text::int(8)).map(|(_, s): (&str, &str)| {
        ParsedNumber::Oct(u64::from_str_radix(s, 8).unwrap_or(u64::MAX))
    });

    bin.or(hex).or(oct).or(int)
}
