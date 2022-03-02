use druid::image::math::Rect;
use druid::image::{DynamicImage, GenericImageView, GenericImage, SubImage};

pub trait TexBank {

    // gets the texture atlas for reading
    fn get_atlas(&self) -> &DynamicImage;

    // gets the texture atlas for writing
    fn get_atlas_mut(&mut self) -> &mut DynamicImage;

    // gets the rectangles describing the location of every texture in atlas for reading
    fn get_region_descriptors(&self) -> &Vec<Rect>;

    // gets the rectangles describing the location of every texture in atlas for writing
    fn get_region_descriptors_mut(&mut self) -> &mut Vec<Rect>;

    // adds new texture to a texture atlas
    fn add_texture(&mut self, tex: DynamicImage) -> usize {
        // adds the texture to the atlas, returns the index of its rectangle descriptor
        let (mut width, mut height) = (tex.width(), tex.height());
        let atlas = self.get_atlas_mut();
        let atlas_height = atlas.height();

        if atlas_height.abs_diff(width) < atlas_height.abs_diff(height) {
            tex.rotate90();

            let tmp = width;
            width = height;
            height = tmp;
        }

        let mut tmp = DynamicImage::new_rgba8(atlas.width() + width, atlas_height.max(height));
        tmp.copy_from(atlas, 0,0);
        tmp.copy_from(&tex, atlas.width(), atlas_height);

        *atlas = tmp;

        let regions = self.get_region_descriptors_mut();
        regions.push(
            Rect {
                x: atlas.width(),
                y: 0,
                width,
                height: atlas_height.max(height),
            }
        );

        return regions.len() - 1;
    }

    fn get_texture_mut(&mut self, id: usize) -> Option<SubImage<&mut DynamicImage>> {

        let atlas = self.get_atlas_mut();
        let region = self.get_region_descriptors().get(id)?;

        Some(atlas.sub_image(region.x, region.y, region.width, region.height))

    }
}