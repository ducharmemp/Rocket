use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use devise::{syn, Spanned, SpanWrapped, Result, FromMeta, Diagnostic};
use devise::ext::{SpanDiagnosticExt, TypeExt};
use indexmap::IndexSet;

use crate::proc_macro_ext::{Diagnostics, StringLit};
use crate::syn_ext::IdentExt;
use crate::proc_macro2::{TokenStream, Span};
use crate::http_codegen::{Method, MediaType, RoutePath, DataSegment, Optional};
use crate::attribute::segments::{Source, Kind, Segment};
use crate::syn::{Attribute, parse::Parser};

use crate::{URI_MACRO_PREFIX, ROCKET_PARAM_PREFIX};


// What do I want to do?
// 1. Generate a wrapping function that does 2 things
//     a. Returns a WebsocketResponse item that completes the handshake
//     b. Spawns off a task after the response that calls the provided function

// pub fn websocket_attribute<M: Into<Option<crate::http::Method>>>(
//     method: M,
//     args: proc_macro::TokenStream,
//     input: proc_macro::TokenStream
// ) -> TokenStream {
//     let result = match method.into() {
//         Some(method) => incomplete_route(method, args.into(), input.into()),
//         None => complete_route(args.into(), input.into())
//     };

//     result.unwrap_or_else(|diag| diag.emit_as_item_tokens())
// }
