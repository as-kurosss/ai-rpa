    // selector.rs

    use anyhow::anyhow;
    use anyhow::Result;
    use uiautomation::{
        UIAutomation,
        UIElement,
        types::ControlType
    };

    /// Способы поиска элементов в UI
    #[derive(Debug, Clone)]
    pub enum Selector {
        /// По имени класса окна (надежно, не зависит от языка)
        Classname(String),

        /// По типу элемента (Button, Edit, Window...)
        ControlType(ControlType),

        /// По имени элемента (заголовок, текст - зависит от локализации!)
        Name(String),

        /// По частичному вхождению в имя (гибче, чем Name)
        NameContains(String),

        /// Комбинация условий (ИЛИ)
        Or(Vec<Selector>),
    }

    impl Selector {
        /// Находит первый элемент, соответствующий селектору
        pub fn find(&self, automation: &UIAutomation, root: &UIElement) -> Result<UIElement, anyhow::Error> {
            match self {
                Selector::Classname(classname) => {
                    let element = automation.create_matcher()
                        .from_ref(root)
                        .classname(classname)
                        .find_first()
                        .map_err(|e| anyhow!("Element not found: classname={}: {}", classname, e))?;
                    Ok(element)
                }

                Selector::ControlType(control_type) => {
                    let element = automation.create_matcher()
                        .control_type(*control_type)
                        .find_first()
                        .map_err(|e| anyhow!("Element not found: control_type={:?}: {}", control_type, e))?;
                    Ok(element)
                }

                Selector::Name(name) => {
                    let element = automation.create_matcher()
                        .name(name)
                        .find_first()
                        .map_err(|e| anyhow!("Element not found: name={}: {}", name, e))?;
                    Ok(element)
                }

                Selector::NameContains(substring) => {
                    let all = automation.create_matcher()
                        .from_ref(root)
                        .find_all()
                        .map_err(|e| anyhow!("Failed to search elements: {}", e))?;

                    for el in &all {
                        if let Ok(name) = el.get_name() {
                            if name.contains(substring.as_str()) {
                                return Ok(el.clone());
                            }
                        }
                    }
                    Err(anyhow!("Element not found: name contains '{}'", substring))
                }

                Selector::Or(selectors) => {
                    for selector in selectors {
                        if let Ok(element) = selector.find(automation, root) {
                            return Ok(element);
                        }
                    }
                    Err(anyhow!("Element not found: {:?}", selectors))
                }
            }   
        }
    }