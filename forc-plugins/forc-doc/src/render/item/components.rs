use crate::{
    doc::module::ModuleInfo,
    render::{
        constant::IDENTITY, item::context::ItemContext, sidebar::*, title::DocBlockTitle, DocStyle,
        Renderable,
    },
    RenderPlan,
};
use anyhow::Result;
use horrorshow::{box_html, Raw, RenderBox};
use sway_core::language::ty::TyDecl;
use sway_types::BaseIdent;

/// All necessary components to render the header portion of
/// the item html doc.
#[derive(Clone, Debug)]
pub(crate) struct ItemHeader {
    pub(crate) module_info: ModuleInfo,
    pub(crate) friendly_name: &'static str,
    pub(crate) item_name: BaseIdent,
}
impl Renderable for ItemHeader {
    /// Basic HTML header component
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let ItemHeader {
            module_info,
            friendly_name,
            item_name,
        } = self;

        let favicon = module_info.to_html_shorthand_path_string("assets/sway-logo.svg");
        let normalize = module_info.to_html_shorthand_path_string("assets/normalize.css");
        let swaydoc = module_info.to_html_shorthand_path_string("assets/swaydoc.css");
        let ayu = module_info.to_html_shorthand_path_string("assets/ayu.css");
        let ayu_hjs = module_info.to_html_shorthand_path_string("assets/ayu.min.css");

        Ok(box_html! {
            head {
                meta(charset="utf-8");
                meta(name="viewport", content="width=device-width, initial-scale=1.0");
                meta(name="generator", content="swaydoc");
                meta(
                    name="description",
                    content=format!(
                        "API documentation for the Sway `{}` {} in `{}`.",
                        item_name.as_str(), friendly_name, module_info.location(),
                    )
                );
                meta(name="keywords", content=format!("sway, swaylang, sway-lang, {}", item_name.as_str()));
                link(rel="icon", href=favicon);
                title: format!("{} in {} - Sway", item_name.as_str(), module_info.location());
                link(rel="stylesheet", type="text/css", href=normalize);
                link(rel="stylesheet", type="text/css", href=swaydoc, id="mainThemeStyle");
                link(rel="stylesheet", type="text/css", href=ayu);
                link(rel="stylesheet", href=ayu_hjs);
                // TODO: Add links for fonts
            }
        })
    }
}

/// All necessary components to render the body portion of
/// the item html doc. Many parts of the HTML body structure will be the same
/// for each item, but things like struct fields vs trait methods will be different.
#[derive(Clone, Debug)]
pub(crate) struct ItemBody {
    pub(crate) module_info: ModuleInfo,
    pub(crate) ty_decl: TyDecl,
    /// The item name varies depending on type.
    /// We store it during info gathering to avoid
    /// multiple match statements.
    pub(crate) item_name: BaseIdent,
    pub(crate) code_str: String,
    pub(crate) attrs_opt: Option<String>,
    pub(crate) item_context: ItemContext,
}
impl SidebarNav for ItemBody {
    fn sidebar(&self) -> Sidebar {
        let style = DocStyle::Item {
            title: Some(self.ty_decl.as_block_title()),
            name: Some(self.item_name.clone()),
        };
        Sidebar::new(
            None,
            style,
            self.module_info.clone(),
            self.item_context.to_doclinks(),
        )
    }
}
impl Renderable for ItemBody {
    /// HTML body component
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let sidebar = self.sidebar();
        let ItemBody {
            module_info,
            ty_decl,
            item_name,
            code_str,
            attrs_opt,
            item_context,
        } = self;

        let decl_ty = ty_decl.doc_name();
        let block_title = ty_decl.as_block_title();
        let sidebar = sidebar.render(render_plan.clone())?;
        let item_context = (item_context.context_opt.is_some())
            .then(|| -> Result<Box<dyn RenderBox>> { item_context.render(render_plan.clone()) });
        let sway_hjs = module_info.to_html_shorthand_path_string("assets/highlight.js");
        let rendered_module_anchors = module_info.get_anchors()?;

        Ok(box_html! {
            body(class=format!("swaydoc {decl_ty}")) {
                : sidebar;
                // this is the main code block
                main {
                    div(class="width-limiter") {
                        // div(class="sub-container") {
                        //     nav(class="sub") {
                        //         form(class="search-form") {
                        //             div(class="search-container") {
                        //                 span;
                        //                 input(
                        //                     class="search-input",
                        //                     name="search",
                        //                     autocomplete="off",
                        //                     spellcheck="false",
                        //                     // TODO: https://github.com/FuelLabs/sway/issues/3480
                        //                     placeholder="Searchbar unimplemented, see issue #3480...",
                        //                     type="search"
                        //                 );
                        //                 div(id="help-button", title="help", tabindex="-1") {
                        //                     button(type="button") { : "?" }
                        //                 }
                        //             }
                        //         }
                        //     }
                        // }
                        section(id="main-content", class="content") {
                            div(class="main-heading") {
                                h1(class="fqn") {
                                    span(class="in-band") {
                                        : format!("{} ", block_title.item_title_str());
                                        @ for anchor in rendered_module_anchors {
                                            : Raw(anchor);
                                        }
                                        a(class=&decl_ty, href=IDENTITY) {
                                            : item_name.as_str();
                                        }
                                    }
                                }
                            }
                            div(class="docblock item-decl") {
                                pre(class=format!("sway {}", &decl_ty)) {
                                    code { : code_str; }
                                }
                            }
                            @ if attrs_opt.is_some() {
                                // expand or hide description of main code block
                                details(class="swaydoc-toggle top-doc", open) {
                                    summary(class="hideme") {
                                        span { : "Expand description" }
                                    }
                                    // this is the description
                                    div(class="docblock") {
                                        : Raw(attrs_opt.unwrap())
                                    }
                                }
                            }
                            @ if item_context.is_some() {
                                : item_context.unwrap();
                            }
                        }
                    }
                }
                script(src=sway_hjs);
                script {
                    : "hljs.highlightAll();";
                }
            }
        })
    }
}
