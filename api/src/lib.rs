pub mod api {
    tonic::include_proto!("blog");
}

pub use api::*;
