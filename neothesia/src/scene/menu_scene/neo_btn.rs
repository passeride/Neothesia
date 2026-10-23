pub struct NeoBtn {
    id: Option<nuon::Id>,
    size: nuon::Size,
    label: String,
    icon: String,
}

impl NeoBtn {
    pub fn new() -> Self {
        Self {
            id: None,
            size: Default::default(),
            label: Default::default(),
            icon: Default::default(),
        }
    }

    #[allow(unused)]
    pub fn id(mut self, id: impl Into<nuon::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height).into();
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn build(&self, ui: &mut nuon::Ui) -> bool {
        let w = self.size.width;
        let h = self.size.height;

        let id = if let Some(id) = self.id {
            id
        } else if self.icon.is_empty() {
            nuon::Id::hash(&self.label)
        } else {
            nuon::Id::hash(&self.icon)
        };

        let event = nuon::click_area(id).size(w, h).build(ui);

        let bg = if event.is_hovered() || event.is_pressed() {
            ui.theme.hover
        } else {
            ui.theme.button
        };
        nuon::quad()
            .size(w, h)
            .color(bg)
            .border_radius([12.0; 4])
            .border_color(ui.theme.border)
            .border_width(1.0)
            .build(ui);

        let label = nuon::label()
            .size(self.size.width, self.size.height)
            .font_size(24.0);
        if self.icon.is_empty() {
            label.text(&self.label).build(ui);
        } else {
            label.icon(&self.icon).build(ui);
        }

        event.is_clicked()
    }
}

pub fn neo_btn() -> NeoBtn {
    NeoBtn::new()
}

pub fn neo_btn_icon(ui: &mut nuon::Ui, w: f32, h: f32, icon: &str) -> bool {
    NeoBtn::new().size(w, h).icon(icon).build(ui)
}
