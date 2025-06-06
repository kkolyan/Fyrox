// Copyright (c) 2019-present Dmitry Stepanov and Fyrox Engine contributors.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Search bar widget is a text box with a "clear text" button. It is used as an input field for search functionality.
//! Keep in mind, that it does **not** provide any built-in searching functionality by itself! See [`SearchBar`] docs
//! for more info and usage examples.

#![warn(missing_docs)]

use crate::style::resource::StyleResourceExt;
use crate::style::Style;
use crate::widget::WidgetMessage;
use crate::{
    border::BorderBuilder,
    brush::Brush,
    button::{ButtonBuilder, ButtonMessage},
    core::{
        algebra::Vector2, color::Color, pool::Handle, reflect::prelude::*, type_traits::prelude::*,
        uuid_provider, variable::InheritableVariable, visitor::prelude::*,
    },
    decorator::DecoratorBuilder,
    define_constructor, define_widget_deref,
    grid::{Column, GridBuilder, Row},
    message::{MessageDirection, UiMessage},
    text::TextMessage,
    text_box::{TextBoxBuilder, TextCommitMode},
    utils::make_cross_primitive,
    vector_image::{Primitive, VectorImageBuilder},
    widget::{Widget, WidgetBuilder},
    BuildContext, Control, HorizontalAlignment, Thickness, UiNode, UserInterface,
    VerticalAlignment,
};

use fyrox_graph::constructor::{ConstructorProvider, GraphNodeConstructor};
use std::ops::{Deref, DerefMut};

/// A set of messages that can be used to get the state of a search bar.
#[derive(Debug, Clone, PartialEq)]
pub enum SearchBarMessage {
    /// Emitted when a user types something in the search bar.
    Text(String),
}

impl SearchBarMessage {
    define_constructor!(
        /// Creates [`SearchBarMessage::Text`] message.
        SearchBarMessage:Text => fn text(String), layout: false
    );
}

/// Search bar widget is a text box with a "clear text" button. It is used as an input field for search functionality.
/// Keep in mind, that it does **not** provide any built-in searching functionality by itself, you need to implement
/// it manually. This widget provides a "standard" looking search bar with very little functionality.
///
/// ## Examples
///
/// ```rust
/// # use fyrox_ui::{
/// #     core::pool::Handle,
/// #     message::UiMessage,
/// #     searchbar::{SearchBarBuilder, SearchBarMessage},
/// #     widget::WidgetBuilder,
/// #     BuildContext, UiNode,
/// # };
/// #
/// fn create_search_bar(ctx: &mut BuildContext) -> Handle<UiNode> {
///     SearchBarBuilder::new(WidgetBuilder::new()).build(ctx)
/// }
///
/// // Somewhere in a UI message loop:
/// fn handle_ui_message(my_search_bar: Handle<UiNode>, message: &UiMessage) {
///     // Catch the moment when the search text has changed and do the actual searching.
///     if let Some(SearchBarMessage::Text(search_text)) = message.data() {
///         if message.destination() == my_search_bar {
///             let items = ["foo", "bar", "baz"];
///
///             println!(
///                 "{} found at {:?} position",
///                 search_text,
///                 items.iter().position(|i| *i == search_text)
///             );
///         }
///     }
/// }
/// ```
#[derive(Default, Clone, Visit, Reflect, Debug, ComponentProvider)]
#[reflect(derived_type = "UiNode")]
pub struct SearchBar {
    /// Base widget of the search bar.
    pub widget: Widget,
    /// A handle of a text box widget used for text input.
    pub text_box: InheritableVariable<Handle<UiNode>>,
    /// A handle of a button, that is used to clear the text.
    pub clear: InheritableVariable<Handle<UiNode>>,
}

impl ConstructorProvider<UiNode, UserInterface> for SearchBar {
    fn constructor() -> GraphNodeConstructor<UiNode, UserInterface> {
        GraphNodeConstructor::new::<Self>()
            .with_variant("Search Bar", |ui| {
                SearchBarBuilder::new(WidgetBuilder::new().with_name("Search Bar"))
                    .build(&mut ui.build_ctx())
                    .into()
            })
            .with_group("Input")
    }
}

define_widget_deref!(SearchBar);

uuid_provider!(SearchBar = "23db1179-0e07-493d-98fd-2b3c0c795215");

impl Control for SearchBar {
    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);

        if message.destination() == self.handle && message.direction() == MessageDirection::ToWidget
        {
            if let Some(SearchBarMessage::Text(text)) = message.data() {
                ui.send_message(TextMessage::text(
                    *self.text_box,
                    MessageDirection::ToWidget,
                    text.clone(),
                ));
            } else if let Some(WidgetMessage::Focus) = message.data() {
                ui.send_message(WidgetMessage::focus(
                    *self.text_box,
                    MessageDirection::ToWidget,
                ));
            }
        }

        if message.destination() == *self.clear {
            if let Some(ButtonMessage::Click) = message.data() {
                ui.send_message(SearchBarMessage::text(
                    self.handle,
                    MessageDirection::ToWidget,
                    String::new(),
                ));
            }
        }

        if message.destination() == *self.text_box
            && message.direction() == MessageDirection::FromWidget
        {
            if let Some(TextMessage::Text(text)) = message.data() {
                ui.send_message(SearchBarMessage::text(
                    self.handle,
                    MessageDirection::FromWidget,
                    text.clone(),
                ));
            }
        }
    }
}

/// Search bar builder creates [`SearchBar`] widget instances and adds them to the user interface.
pub struct SearchBarBuilder {
    widget_builder: WidgetBuilder,
}

impl SearchBarBuilder {
    /// Creates a new builder instance.
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self { widget_builder }
    }

    /// Finishes search bar building and adds the new instance to the user interface.
    pub fn build(mut self, ctx: &mut BuildContext) -> Handle<UiNode> {
        // Focusing the search bar itself is useless, so we're taking the tab index from the inner
        // widget builder and transfer it to the inner text box.
        let tab_index = self.widget_builder.tab_index.take();

        let text_box;
        let clear;
        let content = BorderBuilder::new(
            WidgetBuilder::new()
                .with_foreground(ctx.style.property(Style::BRUSH_LIGHT))
                .with_background(ctx.style.property(Style::BRUSH_DARKER))
                .with_child(
                    GridBuilder::new(
                        WidgetBuilder::new()
                            .with_child(
                                VectorImageBuilder::new(
                                    WidgetBuilder::new()
                                        .with_clip_to_bounds(false)
                                        .with_width(12.0)
                                        .with_height(12.0)
                                        .with_vertical_alignment(VerticalAlignment::Center)
                                        .with_foreground(ctx.style.property(Style::BRUSH_BRIGHT))
                                        .with_margin(Thickness {
                                            left: 4.0,
                                            top: 2.0,
                                            right: 0.0,
                                            bottom: 0.0,
                                        }),
                                )
                                .with_primitives(vec![
                                    Primitive::WireCircle {
                                        center: Vector2::new(4.0, 4.0),
                                        radius: 4.0,
                                        thickness: 1.5,
                                        segments: 16,
                                    },
                                    Primitive::Line {
                                        begin: Vector2::new(6.0, 6.0),
                                        end: Vector2::new(11.0, 11.0),
                                        thickness: 1.5,
                                    },
                                ])
                                .build(ctx),
                            )
                            .with_child({
                                text_box = TextBoxBuilder::new(
                                    WidgetBuilder::new()
                                        .with_tab_index(tab_index)
                                        .on_column(1)
                                        .with_margin(Thickness::uniform(1.0)),
                                )
                                .with_text_commit_mode(TextCommitMode::Immediate)
                                .with_vertical_text_alignment(VerticalAlignment::Center)
                                .build(ctx);
                                text_box
                            })
                            .with_child({
                                clear = ButtonBuilder::new(
                                    WidgetBuilder::new()
                                        .with_width(18.0)
                                        .with_height(18.0)
                                        .on_column(2),
                                )
                                .with_back(
                                    DecoratorBuilder::new(
                                        BorderBuilder::new(WidgetBuilder::new())
                                            .with_pad_by_corner_radius(false)
                                            .with_corner_radius(4.0f32.into()),
                                    )
                                    .with_normal_brush(Brush::Solid(Color::TRANSPARENT).into())
                                    .build(ctx),
                                )
                                .with_content(
                                    VectorImageBuilder::new(
                                        WidgetBuilder::new()
                                            .with_horizontal_alignment(HorizontalAlignment::Center)
                                            .with_vertical_alignment(VerticalAlignment::Center)
                                            .with_height(8.0)
                                            .with_width(8.0)
                                            .with_foreground(
                                                ctx.style.property(Style::BRUSH_BRIGHTEST),
                                            ),
                                    )
                                    .with_primitives(make_cross_primitive(8.0, 2.0))
                                    .build(ctx),
                                )
                                .build(ctx);
                                clear
                            }),
                    )
                    .add_row(Row::stretch())
                    .add_column(Column::auto())
                    .add_column(Column::stretch())
                    .add_column(Column::auto())
                    .build(ctx),
                ),
        )
        .with_corner_radius(4.0f32.into())
        .with_pad_by_corner_radius(false)
        .with_stroke_thickness(Thickness::uniform(1.0).into())
        .build(ctx);

        let search_bar = SearchBar {
            widget: self.widget_builder.with_child(content).build(ctx),
            text_box: text_box.into(),
            clear: clear.into(),
        };

        ctx.add_node(UiNode::new(search_bar))
    }
}

#[cfg(test)]
mod test {
    use crate::selector::SelectorBuilder;
    use crate::{test::test_widget_deletion, widget::WidgetBuilder};

    #[test]
    fn test_deletion() {
        test_widget_deletion(|ctx| SelectorBuilder::new(WidgetBuilder::new()).build(ctx));
    }
}
