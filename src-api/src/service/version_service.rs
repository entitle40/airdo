pub fn get_build_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_build_time() -> &'static str {
    env!("VERGEN_BUILD_TIMESTAMP")
}

pub fn get_git_sha() -> &'static str {
    env!("VERGEN_GIT_SHA")
}