use crate::eq::Expression;
use std::sync::Arc;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum ExpKind {
    #[default]
    Value,
    BasicOp(char),
    Neg,
    Sqrt,
    Power,
}

#[derive(Default, Debug, Clone)]
pub struct ExpressionWidget {
    kind: ExpKind,
    op1: Option<Box<ExpressionWidget>>,
    op2: Option<Box<ExpressionWidget>>,
    str: Option<String>,

    layout: Option<Arc<egui::Galley>>,
    cached_bounds: Option<egui::Vec2>,
}

impl ExpressionWidget {
    fn layout_ui(
        &mut self,
        painter: &egui::Painter,
        font_id: egui::FontId,
        text_color: egui::Color32,
    ) {
        match self.kind {
            ExpKind::Value => {
                self.layout = Some(painter.layout_no_wrap(
                    self.str.as_ref().unwrap().clone(),
                    font_id.clone(),
                    text_color,
                ));
            }
            ExpKind::BasicOp(c) => {
                self.op1
                    .as_mut()
                    .unwrap()
                    .layout_ui(painter, font_id.clone(), text_color);
                self.op2
                    .as_mut()
                    .unwrap()
                    .layout_ui(painter, font_id.clone(), text_color);
                self.layout =
                    Some(painter.layout_no_wrap(format!(" {} ", c), font_id.clone(), text_color));
            }
            ExpKind::Neg => {
                self.op1
                    .as_mut()
                    .unwrap()
                    .layout_ui(painter, font_id.clone(), text_color);
                self.layout =
                    Some(painter.layout_no_wrap(" -".to_string(), font_id.clone(), text_color));
            }
            ExpKind::Sqrt => {
                self.op1
                    .as_mut()
                    .unwrap()
                    .layout_ui(painter, font_id.clone(), text_color);
            }
            ExpKind::Power => {
                self.op1
                    .as_mut()
                    .unwrap()
                    .layout_ui(painter, font_id.clone(), text_color);
                self.op2
                    .as_mut()
                    .unwrap()
                    .layout_ui(painter, font_id.clone(), text_color);
                self.layout = Some(painter.layout_no_wrap(" ".into(), font_id.clone(), text_color));
            }
        }

        self.cached_bounds = Some(self.total_bounds());
    }

    fn total_bounds(&self) -> egui::Vec2 {
        if let Some(cache) = self.cached_bounds {
            return cache;
        }

        match self.kind {
            ExpKind::Value => self.layout.as_ref().unwrap().size(),
            ExpKind::BasicOp(c) => {
                let s1 = self.op1.as_ref().unwrap().total_bounds();
                let s2 = self.op2.as_ref().unwrap().total_bounds();

                match c {
                    '/' => egui::Vec2 {
                        x: s1.x.max(s2.x),
                        y: 2. + s1.y + s2.y,
                    },
                    _ => {
                        let self_sz = self.layout.as_ref().unwrap().size();
                        egui::Vec2 {
                            x: s1.x + s2.x + self_sz.x,
                            y: s1.y.max(s2.y).max(self_sz.y),
                        }
                    }
                }
            }
            ExpKind::Neg => {
                let s1 = self.op1.as_ref().unwrap().total_bounds();
                let self_sz = self.layout.as_ref().unwrap().size();
                egui::Vec2 {
                    x: s1.x + self_sz.x,
                    y: s1.y.max(self_sz.y),
                }
            }
            ExpKind::Sqrt => {
                let s1 = self.op1.as_ref().unwrap().total_bounds();
                egui::Vec2 {
                    x: s1.x + 14.,
                    y: s1.y + 3.,
                }
            }
            ExpKind::Power => {
                let s1 = self.op1.as_ref().unwrap().total_bounds();
                let s2 = self.op2.as_ref().unwrap().total_bounds();

                let self_sz = self.layout.as_ref().unwrap().size();
                egui::Vec2 {
                    x: s1.x + s2.x + self_sz.x,
                    y: s1.y.max(s2.y.max(self_sz.y) + 7.),
                }
            }
        }
    }

    fn ui(&self, pos: egui::Pos2, painter: &egui::Painter, stroke: egui::Stroke) -> egui::Rect {
        match self.kind {
            ExpKind::Value => {
                painter.add(egui::Shape::galley(
                    pos,
                    self.layout.as_ref().unwrap().clone(),
                ));
                egui::Rect::from_min_size(pos, self.layout.as_ref().unwrap().size())
            }
            ExpKind::BasicOp(c) => match c {
                '/' => {
                    let (b1, b2) = (
                        self.op1.as_ref().unwrap().total_bounds(),
                        self.op2.as_ref().unwrap().total_bounds(),
                    );
                    let width = b1.x.max(b2.x);

                    let op1 = self.op1.as_ref().unwrap().ui(
                        egui::Pos2 {
                            x: pos.x + ((width - b1.x) / 2.).max(0.),
                            y: pos.y,
                        },
                        painter,
                        stroke,
                    );

                    painter.hline(
                        pos.x..=(pos.x + width),
                        painter.round_to_pixel(op1.max.y + 1.),
                        stroke,
                    );

                    let op2 = self.op2.as_ref().unwrap().ui(
                        egui::Pos2 {
                            x: pos.x + ((width - b2.x) / 2.).max(0.),
                            y: op1.max.y + stroke.width + 2.,
                        },
                        painter,
                        stroke,
                    );
                    egui::Rect {
                        min: pos,
                        max: op2.max,
                    }
                }

                _ => {
                    let (b1, b2) = (
                        self.op1.as_ref().unwrap().total_bounds(),
                        self.op2.as_ref().unwrap().total_bounds(),
                    );
                    let height = b1.y.max(b2.y);

                    let op1 = self.op1.as_ref().unwrap().ui(
                        egui::Pos2 {
                            x: pos.x,
                            y: pos.y + ((height - b1.y) / 2.).max(0.),
                        },
                        painter,
                        stroke,
                    );
                    painter.add(egui::Shape::galley(
                        egui::Pos2 {
                            x: pos.x + b1.x,
                            y: pos.y
                                + ((height - self.layout.as_ref().unwrap().size().y) / 2.).max(0.),
                        },
                        self.layout.as_ref().unwrap().clone(),
                    ));
                    let op2 = self.op2.as_ref().unwrap().ui(
                        egui::Pos2 {
                            x: pos.x + b1.x + self.layout.as_ref().unwrap().rect.width(),
                            y: pos.y + ((height - b2.y) / 2.).max(0.),
                        },
                        painter,
                        stroke,
                    );

                    egui::Rect {
                        min: pos,
                        max: egui::Pos2 {
                            x: pos.x + b1.x + b2.x + self.layout.as_ref().unwrap().rect.width(),
                            y: pos.y + height,
                        },
                    }
                }
            },
            ExpKind::Neg => {
                painter.add(egui::Shape::galley(
                    egui::Pos2 { x: pos.x, y: pos.y },
                    self.layout.as_ref().unwrap().clone(),
                ));
                let op1 = self.op1.as_ref().unwrap().ui(
                    egui::Pos2 {
                        x: pos.x + self.layout.as_ref().unwrap().rect.width(),
                        y: pos.y,
                    },
                    painter,
                    stroke,
                );

                egui::Rect {
                    min: pos,
                    max: op1.max,
                }
            }
            ExpKind::Sqrt => {
                // painter.add(egui::Shape::galley(
                //     egui::Pos2 { x: pos.x, y: pos.y },
                //     self.layout.as_ref().unwrap().clone(),
                // ));
                let op1 = self.op1.as_ref().unwrap().ui(
                    egui::Pos2 {
                        x: pos.x + 12.,
                        y: pos.y + 1.,
                    },
                    painter,
                    stroke,
                );

                painter.line_segment(
                    [
                        egui::Pos2 {
                            x: pos.x,
                            y: pos.y + op1.height() / 2. - 1.,
                        },
                        egui::Pos2 {
                            x: pos.x + 3.,
                            y: pos.y + op1.height() / 2. - 1.,
                        },
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        egui::Pos2 {
                            x: pos.x + 3.,
                            y: pos.y + op1.height() / 2. - 1.,
                        },
                        egui::Pos2 {
                            x: pos.x + 5.,
                            y: pos.y + op1.height() + 1.,
                        },
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        egui::Pos2 {
                            x: pos.x + 5.,
                            y: pos.y + op1.height() + 1.,
                        },
                        egui::Pos2 {
                            x: pos.x + 9.,
                            y: pos.y + 1.,
                        },
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        egui::Pos2 {
                            x: pos.x + 9.,
                            y: pos.y + 1.,
                        },
                        egui::Pos2 {
                            x: pos.x + op1.width() + 14.,
                            y: pos.y + 1.,
                        },
                    ],
                    stroke,
                );

                egui::Rect {
                    min: pos,
                    max: op1.max,
                }
            }
            ExpKind::Power => {
                let (b1, b2, sz) = (
                    self.op1.as_ref().unwrap().total_bounds(),
                    self.op2.as_ref().unwrap().total_bounds(),
                    self.layout.as_ref().unwrap().size(),
                );
                let height = b1.y.max(b2.y.max(sz.y) + 7.);

                let op1 = self.op1.as_ref().unwrap().ui(
                    egui::Pos2 {
                        x: pos.x,
                        y: pos.y + 7.,
                    },
                    painter,
                    stroke,
                );
                painter.add(egui::Shape::galley(
                    egui::Pos2 {
                        x: pos.x + b1.x,
                        y: pos.y,
                    },
                    self.layout.as_ref().unwrap().clone(),
                ));
                let op2 = self.op2.as_ref().unwrap().ui(
                    egui::Pos2 {
                        x: pos.x + b1.x + self.layout.as_ref().unwrap().rect.width(),
                        y: pos.y,
                    },
                    painter,
                    stroke,
                );

                egui::Rect {
                    min: pos,
                    max: egui::Pos2 {
                        x: pos.x + b1.x + b2.x + self.layout.as_ref().unwrap().rect.width(),
                        y: pos.y + height,
                    },
                }
            }
        }
    }
}

impl From<&Expression> for ExpressionWidget {
    fn from(e: &Expression) -> Self {
        match e {
            Expression::Variable(v) => ExpressionWidget {
                kind: ExpKind::Value,
                str: Some(format!("{}", &v).to_owned()),
                ..ExpressionWidget::default()
            },
            Expression::Rational(_, _) | Expression::Integer(_) => ExpressionWidget {
                kind: ExpKind::Value,
                str: Some(format!("{}", &e).to_owned()),
                ..ExpressionWidget::default()
            },
            Expression::Equal(a, b) => ExpressionWidget {
                kind: ExpKind::BasicOp('='),
                op1: Some(Box::new(a.as_ref().into())),
                op2: Some(Box::new(b.as_ref().into())),
                ..ExpressionWidget::default()
            },
            Expression::Sum(a, b) => ExpressionWidget {
                kind: ExpKind::BasicOp('+'),
                op1: Some(Box::new(a.as_ref().into())),
                op2: Some(Box::new(b.as_ref().into())),
                ..ExpressionWidget::default()
            },
            Expression::Difference(a, b) => ExpressionWidget {
                kind: ExpKind::BasicOp('-'),
                op1: Some(Box::new(a.as_ref().into())),
                op2: Some(Box::new(b.as_ref().into())),
                ..ExpressionWidget::default()
            },
            Expression::Product(a, b) => {
                if let (Expression::Integer(c), Expression::Variable(v)) = (a.as_ref(), b.as_ref())
                {
                    ExpressionWidget {
                        kind: ExpKind::Value,
                        str: Some(format!("{}{}", c, v).to_owned()),
                        ..ExpressionWidget::default()
                    }
                } else {
                    ExpressionWidget {
                        kind: ExpKind::BasicOp('·'),
                        op1: Some(Box::new(a.as_ref().into())),
                        op2: Some(Box::new(b.as_ref().into())),
                        ..ExpressionWidget::default()
                    }
                }
            }
            Expression::Quotient(a, b) => ExpressionWidget {
                kind: ExpKind::BasicOp('/'),
                op1: Some(Box::new(a.as_ref().into())),
                op2: Some(Box::new(b.as_ref().into())),
                ..ExpressionWidget::default()
            },
            Expression::Neg(a) => ExpressionWidget {
                kind: ExpKind::Neg,
                op1: Some(Box::new(a.as_ref().into())),
                ..ExpressionWidget::default()
            },
            Expression::Sqrt(a, _) => ExpressionWidget {
                kind: ExpKind::Sqrt,
                op1: Some(Box::new(a.as_ref().into())),
                ..ExpressionWidget::default()
            },
            Expression::Power(a, b) => ExpressionWidget {
                kind: ExpKind::Power,
                op1: Some(Box::new(a.as_ref().into())),
                op2: Some(Box::new(b.as_ref().into())),
                ..ExpressionWidget::default()
            },
        }
    }
}

pub struct SizedExpression(ExpressionWidget, egui::Vec2);

impl SizedExpression {
    pub fn layout(ui: &mut egui::Ui, exp: &Expression) -> Self {
        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let text_color = ui.visuals().text_color();

        let mut w: ExpressionWidget = exp.into();
        w.layout_ui(ui.painter(), font_id, text_color);
        let bb = w.total_bounds();
        Self(w, bb)
    }

    pub fn dims(&self) -> egui::Vec2 {
        self.1
    }

    pub fn ui(self, ui: &mut egui::Ui) -> egui::Rect {
        let (rec, _resp) = ui.allocate_at_least(self.1, egui::Sense::click());
        self.0.ui(
            rec.min,
            ui.painter(),
            ui.visuals().widgets.noninteractive.bg_stroke,
        )
    }
}
