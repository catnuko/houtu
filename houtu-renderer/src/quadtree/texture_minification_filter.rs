#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum TextureMinificationFilter {
    Nearest = 0,
    Linear = 1,
    NearestMipmapNearest = 2,
    LinearMipmapNearest = 3,
    NearestMipmapLinear = 4,
    LinearMipmapLinear = 5,
}
impl TextureMinificationFilter {
    pub fn validate(texture_minification_filter: TextureMinificationFilter) -> bool {
        return (texture_minification_filter == TextureMinificationFilter::Nearest
            || texture_minification_filter == TextureMinificationFilter::Linear
            || texture_minification_filter == TextureMinificationFilter::NearestMipmapNearest
            || texture_minification_filter == TextureMinificationFilter::LinearMipmapNearest
            || texture_minification_filter == TextureMinificationFilter::NearestMipmapLinear
            || texture_minification_filter == TextureMinificationFilter::LinearMipmapLinear);
    }
}
