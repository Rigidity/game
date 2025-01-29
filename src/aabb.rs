use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn ray_intersection(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<f32> {
        let t1 = (self.min - ray_origin) / ray_direction;
        let t2 = (self.max - ray_origin) / ray_direction;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_near = t_min.x.max(t_min.y).max(t_min.z);
        let t_far = t_max.x.min(t_max.y).min(t_max.z);

        if t_near > t_far || t_far < 0.0 {
            None
        } else {
            Some(t_near.max(0.0))
        }
    }
}
