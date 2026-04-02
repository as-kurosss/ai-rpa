    // selector.rs

    use anyhow::anyhow;
    use anyhow::Result;
    use uiautomation::{
        UIAutomation,
        UIElement,
        types::TreeScope
    };

    /// Способы поиска элементов в UI
    #[derive(Debug, Clone)]
    pub enum Selector {
        /// По имени класса окна (надежно, не зависит от языка)
        Classname(String),

        /// По типу элемента (Button, Edit, Window...)
        ControlType(uiautomation::ControlType),

        /// По имени элемента (заголовок, текст - зависит от локализации!)
        Name(String),

        /// Комбинация условий (ИЛИ)
        Or(Vec<Selector>),
    }

    impl Selector {
        /// Находит первый элемент, соответствующий селектору
        pub fn find(&self, automation: &UIAutomation, root: &UIElement) -> Result<UIElement> {
            match self {
                Selector::Classname(classname) => {
                    let matcher = automation.create_matcher()
                        .classname(classname)
                        .scope(TreeScope::Subtree);
                    matcher.find_first()
                        .ok_or_else(|| anyhow!("Element not found: classname={}", classname))?;
                }

                Selector::ControlType(control_type) => {
                    let matcher = automation.create_matcher()
                        .control_type(*control_type)
                        .scope(TreeScope::Subtree);
                    matcher.find_first()
                        .ok_or_else(|| anyhow!("Element not found: control_type={:?}", control_type))
                }

                Selector::Name(name) => {
                    let matcher = automation.create_matcher()
                        .name(name)
                        .scope(TreeScope::Subtree);
                    matcher.find_first()
                        .ok_or_else(|| anyhow!("Element not found: name={}", name))
                }

                Selector::Or(selectors) => {
                    for selector in selectors {
                        if let Some(element) = selector.find(automation, root).ok() {
                            return Ok(element);
                        }
                    }
                    Err(anyhow!("Element not found: and={:?}", selectors))
                }

                _ => unimplemented!(),
            }   
        }
    }