use crate::{RootResource, vec3i::Vec3i};
use rand::{rngs::ThreadRng, distributions::uniform::SampleUniform, Rng};

pub fn random_resource(rng: &mut ThreadRng) -> RootResource {
    let rng = generate_random_number(rng);
    if rng > 0.8 {
        return RootResource::Sap;
    }
    if rng > 0.5 {
        return RootResource::Bark;
    }
    RootResource::Wood
}

pub fn random_location(rng: &mut ThreadRng, min: i64, max: i64) -> Vec3i {
    Vec3i::new(generate_random_between(rng, min, max), 0, generate_random_between(rng, min, max))
}

pub fn generate_random_between<T> (rng: &mut ThreadRng, min: T, max: T) -> T
where T: SampleUniform + std::cmp::PartialOrd {
    let range = min..=max;
    rng.gen_range(range)
}

pub fn generate_random_number(rng: &mut ThreadRng, ) -> f32 {
    rng.gen::<f32>()
}
