#![allow(clippy::empty_docs)] // Clippy bug
use js_sys::{Array, Object, Uint32Array};
use pob::TreeSpec;
use sycamore::prelude::{GenericNode, NodeRef};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlElement;

use crate::utils::{from_ref, reflect_set};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    type TreeObj;

    #[wasm_bindgen(method, js_name=tree_load)]
    fn load(this: &TreeObj, data: JsValue);

    #[wasm_bindgen(method, js_name=tree_highlight)]
    fn highlight(this: &TreeObj, data: JsValue);
}

pub struct SvgTree(TreeObj);

impl SvgTree {
    pub fn url(spec: &TreeSpec) -> &'static str {
        match spec.version {
            Some("3_15") | Some("3.15") => "/assets/3.15.svg",
            Some("3_16") | Some("3.16") => "/assets/3.16.svg",
            Some("3_17") | Some("3.17") => "/assets/3.17.svg",
            Some("3_18") | Some("3.18") => "/assets/3.18.svg",
            Some("3_19") | Some("3.19") => "/assets/3.19.svg",
            Some("3_20") | Some("3.20") => "/assets/3.20.svg",
            Some("3_21") | Some("3.21") => "/assets/3.21.svg",
            Some("3_22") | Some("3.22") => "/assets/3.22.svg",
            Some("3_23") | Some("3.23") => "/assets/3.23.svg",
            Some("3_24") | Some("3.24") => "/assets/3.24.svg",
            _ => "/assets/3.25.svg",
        }
    }

    pub fn from_ref<G: GenericNode>(node_ref: &NodeRef<G>) -> Option<Self> {
        let inner = from_ref::<web_sys::HtmlObjectElement>(node_ref)
            .content_window()?
            .unchecked_into::<TreeObj>();

        Some(Self(inner))
    }

    pub fn element(&self) -> HtmlElement {
        self.0.clone().unchecked_into()
    }

    pub fn load(&self, spec: &TreeSpec<'_>) {
        let obj = Object::new();
        reflect_set(&obj, "nodes", Uint32Array::from(spec.nodes));
        reflect_set(&obj, "classId", spec.class_id);
        reflect_set(&obj, "ascendancyId", spec.ascendancy_id);
        reflect_set(&obj, "alternateAscendancyId", spec.alternate_ascendancy_id);

        self.0.load(obj.into());
    }

    pub fn highlight<I, T>(&self, nodes: I)
    where
        I: Iterator<Item = T>,
        T: Into<JsValue>,
    {
        let obj = Array::new();
        for node_id in nodes {
            obj.push(&node_id.into());
        }

        self.0.highlight(obj.into());
    }

    pub fn clear_highlight(&self) {
        self.0.highlight(Array::new().into());
    }
}
