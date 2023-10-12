use chumsky::prelude::*;
use egui_extras::{Column, TableBuilder};
use egui_extras::{Size, StripBuilder};

use crate::eq;

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
                egui::CollapsingHeader::new("Integer representations")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.push_id("numberz", |ui| {
                            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                            let table = TableBuilder::new(ui)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::TOP))
                                .column(Column::exact(120.0))
                                .column(Column::exact(50.0))
                                .column(Column::remainder())
                                .auto_shrink([true, true]);

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

#[derive(Default, Debug)]
pub struct EquationExtra {}

impl EquationExtra {
    fn parse(&self, input: &String) -> Option<eq::Expression> {
        match eq::Expression::parse(input, false) {
            Ok(p) => Some(p),
            Err(_e) => None,
        }
    }
}

impl ImmediateExtra for EquationExtra {
    fn ui(&mut self, input: &String, ctx: &egui::Context, ui: &mut egui::Ui) -> bool {
        use crate::eq::Expression;

        match self.parse(input) {
            Some(eq) => {
                if let Expression::Variable(_) = eq {
                    return false;
                }

                let mut vars = std::collections::BTreeMap::<crate::eq::Variable, usize>::new();
                eq.walk(&mut |e| {
                    if let crate::eq::Expression::Variable(v) = e {
                        match vars.get_mut(&v) {
                            Some(count) => *count += 1,
                            None => {
                                vars.insert(v.clone(), 1);
                            }
                        }
                    }
                    true
                });

                egui::CollapsingHeader::new("Equation")
                    .default_open(true)
                    .show(ui, |ui| {
                        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                        let inp = crate::eqwidget::SizedExpression::layout(ui, &eq);
                        let mut simp = eq.clone();
                        simp.simplify();
                        let simpW = crate::eqwidget::SizedExpression::layout(ui, &simp);
                        let var_eqs: Vec<_> = vars
                            .keys()
                            .map(|var| {
                                let mut eq = simp.clone();
                                match eq.make_subject(&Expression::Variable(var.clone())) {
                                    Ok(Expression::Equal(_, eq2)) => {
                                        let eqW =
                                            crate::eqwidget::SizedExpression::layout(ui, &eq2);

                                        Some((var, eqW, eq2))
                                    }
                                    Err(_) => None,
                                    _ => None,
                                }
                            })
                            .filter(|e| e.is_some())
                            .map(|e| e.unwrap())
                            .collect();

                        let table = TableBuilder::new(ui)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::TOP))
                            .column(Column::exact(120.0))
                            .column(Column::exact(50.0))
                            .column(Column::remainder())
                            .auto_shrink([true, true]);

                        table.body(|mut body| {
                            body.row(inp.dims().y, |mut row| {
                                row.col(|ui| {
                                    ui.strong("Input");
                                });
                                row.col(|ui| {});
                                row.col(|ui| {
                                    inp.ui(ui);
                                });
                            });

                            body.row(simpW.dims().y, |mut row| {
                                row.col(|ui| {
                                    ui.strong("Simplified");
                                });
                                row.col(|ui| {
                                    if ui.small_button("📋").clicked() {
                                        ctx.output_mut(|o| o.copied_text = format!("{}", simp));
                                    }
                                });
                                row.col(|ui| {
                                    simpW.ui(ui);
                                });
                            });
                            if let Expression::Rational(r, true) = &simp {
                                let dec = Expression::Rational(r.clone(), false);
                                body.row(text_height, |mut row| {
                                    row.col(|ui| {
                                        ui.strong("As decimal");
                                    });
                                    row.col(|ui| {
                                        if ui.small_button("📋").clicked() {
                                            ctx.output_mut(|o| o.copied_text = format!("{}", dec));
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", dec));
                                    });
                                });
                            }

                            for (var, eqW, eq) in var_eqs {
                                body.row(eqW.dims().y, |mut row| {
                                    row.col(|ui| {
                                        ui.strong("Rearranged: ".to_string() + var);
                                    });
                                    row.col(|ui| {
                                        if ui.small_button("📋").clicked() {
                                            ctx.output_mut(|o| o.copied_text = format!("{}", eq));
                                        }
                                    });
                                    row.col(|ui| {
                                        eqW.ui(ui);
                                    });
                                });
                            }
                        });
                    });

                true
            }
            None => false,
        }
    }
}
