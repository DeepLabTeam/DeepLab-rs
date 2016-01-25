use conrod::{
    CharacterCache,
    Circle,
    Color,
    Colorable,
    CommonBuilder,
    Dimensions,
    FontSize,
    IndexSlot,
    Labelable,
    Mouse,
    Point,
    Positionable,
    Scalar,
    Text,
    Theme,
    UpdateArgs,
    Widget,
    WidgetKind,
};
use vecmath;

/// The type upon which we'll implement the `Widget` trait.
pub struct NodeBlock<'a, F> {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    common: CommonBuilder,
    /// Optional label string for the button.
    maybe_label: Option<&'a str>,
    /// Optional callback for when the button is pressed. If you want the button to
    /// do anything, this callback must exist.
    maybe_react: Option<F>,
    /// See the Style struct below.
    style: Style,
    /// Whether the button is currently enabled, i.e. whether it responds to
    /// user input.
    enabled: bool
}

/// Represents the unique styling for our NodeBlock widget.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    /// Color of the button.
    pub maybe_color: Option<Color>,
    /// Radius of the button.
    pub maybe_radius: Option<Scalar>,
    /// Color of the button's label.
    pub maybe_label_color: Option<Color>,
    /// Font size of the button's label.
    pub maybe_label_font_size: Option<u32>,
}

/// Represents the unique, cached state for our NodeBlock widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// The current interaction state. See the Interaction enum below. See also
    /// get_new_interaction below, where we define all the logic for transitioning between
    /// interaction states.
    interaction: Interaction,
    /// An index to use for our **Circle** primitive graphics widget.
    circle_idx: IndexSlot,
    /// An index to use for our **Text** primitive graphics widget (for the label).
    text_idx: IndexSlot,

    pos: [f64; 2],
}

/// A `&'static str` that can be used to uniquely identify our widget type.
pub const KIND: WidgetKind = "NodeBlock";

/// A type to keep track of interaction between updates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Drag,
    Selected,
}

impl Interaction {
    /// Alter the widget color depending on the current interaction.
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Normal => color,
            Interaction::Highlighted => color.highlighted(),
            Interaction::Drag => color.highlighted(),
            Interaction::Selected => color.clicked(),
        }
    }
}

/// Check the current interaction with the button. Takes into account whether the mouse is
/// over the button and the previous interaction state.
fn get_new_interaction(is_over: bool, prev: Interaction, mouse: Mouse) -> Interaction {
    use conrod::MouseButtonPosition::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Drag, Selected};
    match (is_over, prev, mouse.left.position) {
        // LMB is down over the button. But the button wasn't Highlighted last
        // update. This means the user clicked somewhere outside the button and
        // moved over the button holding LMB down. We do nothing in this case.
        (true, Normal, Down) => Normal,

        // LMB is down over the button. The button was either Highlighted or Clicked
        // last update. If it was highlighted before, that means the user clicked
        // just now, and we transition to the Clicked state. If it was clicked
        // before, that means the user is still holding LMB down from a previous
        // click, in which case the state remains Clicked.
        (true,  _, Down) => Drag,

        // LMB is down, the mouse is not over the button, but the previous state was
        // Clicked. That means the user clicked the button and then moved the mouse
        // outside the button while holding LMB down. The button stays Clicked.
        (false, Drag, Down) => Drag,

        // Releasing LMB after dragging always transitions to selected
        (_, Drag, Up) => Selected,

        // Stay selected
        (_, Selected, Up) => Selected,

        // Unselect when user clicks outside
        (false, Selected, Down) => Normal,
        
        // Highlight if mouse is over the node and LMB is released
        (true, _, Up) => Highlighted,

        // If none of the above applies, then nothing interesting is happening with
        // this button.
        _ => Normal,
    }
}

/// Return whether or not a given point is over a circle at a given point on a
/// Cartesian plane. We use this to determine whether the mouse is over the button.
pub fn is_over_circ(circ_center: Point, mouse_point: Point, dim: Dimensions) -> bool {
    // Offset vector from the center of the circle to the mouse.
    let offset = vecmath::vec2_sub(mouse_point, circ_center);

    // If the length of the offset vector is less than or equal to the circle's
    // radius, then the mouse is inside the circle. We assume that dim is a square
    // bounding box around the circle, thus 2 * radius == dim[0] == dim[1].
    vecmath::vec2_len(offset) <= dim[0] / 2.0
}

impl<'a, F> NodeBlock<'a, F> {
    /// Create a button context to be built upon.
    pub fn new() -> NodeBlock<'a, F> {
        NodeBlock {
            common: CommonBuilder::new(),
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Set the reaction for the Button. The reaction will be triggered upon release
    /// of the button. Like other Conrod configs, this returns self for chainability.
    pub fn react(mut self, reaction: F) -> Self {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.  Like
    /// other Conrod configs, this returns self for chainability. Allow dead code
    /// because we never call this in the example.
    #[allow(dead_code)]
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }
}

/// A custom Conrod widget must implement the Widget trait. See the **Widget** trait
/// documentation for more details.
impl<'a, F> Widget for NodeBlock<'a, F>
    where F: FnMut()
{
    /// The State struct that we defined above.
    type State = State;

    /// The Style struct that we defined above.
    type Style = Style;

    fn common(&self) -> &CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            interaction: Interaction::Normal,
            circle_idx: IndexSlot::new(),
            text_idx: IndexSlot::new(),
            pos: [0.0, 0.0],
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the button. The state may or may not have changed since
    /// the last update. (E.g. it may have changed because the user moused over the
    /// button.) If the state has changed, return the new state. Else, return None.
    fn update<C: CharacterCache>(mut self, args: UpdateArgs<Self, C>) {
        //self.xy(args.state.state.pos);

        let UpdateArgs { idx, state, rect, mut ui, style, .. } = args;
        let (xy, dim) = rect.xy_dim();
        let maybe_mouse = ui.input().maybe_mouse.map(|mouse| mouse.relative_to(xy));

        // Check whether or not a new interaction has occurred.
        let new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                // Conrod does us a favor by transforming mouse.xy into this widget's
                // local coordinate system. Because mouse.xy is in local coords,
                // we must also pass the circle center in local coords. Thus we pass
                // [0.0, 0.0] as the center.
                //
                // See above where we define is_over_circ.
                let is_over = is_over_circ([0.0, 0.0], mouse.xy, dim);

                // See above where we define get_new_interaction.
                get_new_interaction(is_over, state.view().interaction, mouse)
            },
        };

        // If the mouse was released over the button, react. state.interaction is the
        // button's state as of a moment ago. new_interaction is the updated state as
        // of right now. So this if statement is saying: If the button was clicked a
        // moment ago, and it's now highlighted, then the button has been activated.
        if let (Interaction::Drag, Interaction::Selected) =
            (state.view().interaction, new_interaction)
        {
            // Recall that our NodeBlock struct includes maybe_react, which
            // stores either a reaction function or None. If maybe_react is Some, call
            // the function.
            if let Some(ref mut react) = self.maybe_react {
                react();
            }
        }

        // Here we check to see whether or not our button should capture the mouse.
        match (state.view().interaction, new_interaction) {
            // If the user has pressed the button we capture the mouse.
            (Interaction::Highlighted, Interaction::Drag) => {
                ui.capture_mouse();
            },
            // If the user releases the button, we uncapture the mouse.
            (Interaction::Drag, Interaction::Selected) |
            (Interaction::Selected, Interaction::Normal) => {
                ui.uncapture_mouse();
            },
            _ => (),
        }

        // If the interaction has changed, set the new interaction.
        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        // First, we'll draw the **Circle** with a radius that is half our given width.
        let radius = rect.w() / 2.0;
        let color = new_interaction.color(style.color(ui.theme()));
        let circle_idx = state.view().circle_idx.get(&mut ui);
        Circle::fill(radius)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .set(circle_idx, &mut ui);

        // Now we'll instantiate our label using the **Text** widget.
        let label_color = style.label_color(ui.theme());
        let font_size = style.label_font_size(ui.theme());
        let text_idx = state.view().text_idx.get(&mut ui);
        if let Some(ref label) = self.maybe_label {
            Text::new(label)
                .middle_of(idx)
                .font_size(font_size)
                .graphics_for(idx)
                .color(label_color)
                .set(text_idx, &mut ui);
        }
    }

}

impl Style {
    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_radius: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }
}

/// Provide the chainable color() configuration method.
impl<'a, F> Colorable for NodeBlock<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

/// Provide the chainable label(), label_color(), and label_font_size()
/// configuration methods.
impl<'a, F> Labelable<'a> for NodeBlock<'a, F> {
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }
    fn label_color(mut self, color: Color) -> Self {
        self.style.maybe_label_color = Some(color);
        self
    }
    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_label_font_size = Some(size);
        self
    }
}
