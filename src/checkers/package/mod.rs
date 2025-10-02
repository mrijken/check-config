pub(crate) mod package_absent;
pub(crate) mod package_present;

#[derive(Debug, Clone)]
pub(crate) struct PackageCheck {
    pub(crate) package: String,
}
