use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};




pub struct Camera {
    // pub velocity: Point3<f32>,
    pub position: Point3<f32>,
    pub pitch: Rad<f32>,
    pub yaw: Rad<f32>,
    pub sensitivity: f32,
}


impl Camera {
    
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        // println!("{} {} pitch yaw", self.pitch.0, self.yaw.0);

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw
            ).normalize(),
            Vector3::unit_y(),
        )
    }

    // pub fn get_view_matrix(&self) -> Matrix4<f32> {
    //     // Create translation matrix
    //     let camera_translation = Matrix4::from_translation(self.position);

    //     // Your rotation matrix (must already be Matrix4<f32>)
    //     let camera_rotation = self.get_rotation_matrix();

    //     // Invert the camera transform to get the view matrix
    //     (camera_translation * camera_rotation)
    //         .invert()
    //         .expect("Camera matrix should be invertible")
    // }

    // pub fn get_rotation_matrix(&self) -> Matrix4<f32> {
    //     Matrix4::from_angle_y(Rad(-self.yaw))
    //         * Matrix4::from_angle_x(Rad(self.pitch))
    // }
}