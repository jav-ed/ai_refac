//! Turning a request into the groups the drivers move: which language owns
//! each path (`route`, `referenced`), the limits and the tools every group
//! needs before anything moves (`prepare`, `typescript`, `unavailable`).
pub(super) mod prepare;
mod referenced;
pub(super) mod route;
pub(super) mod typescript;
pub(super) mod unavailable;
