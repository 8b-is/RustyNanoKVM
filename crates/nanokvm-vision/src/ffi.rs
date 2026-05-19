use std::os::raw::{c_int, c_uchar, c_uint, c_ushort};

#[link(name = "kvm")]
unsafe extern "C" {
    pub fn kvmv_init(_debug_info_en: c_uchar);
    pub fn set_venc_auto_recyc(_enable: c_uchar);
    pub fn kvmv_read_img(
        _width: c_ushort,
        _height: c_ushort,
        _type: c_uchar,
        _qlty: c_ushort,
        _pp_kvm_data: *mut *mut c_uchar,
        _p_kvmv_data_size: *mut c_uint,
    ) -> c_int;
    pub fn free_kvmv_data(_pp_kvm_data: *mut *mut c_uchar) -> c_int;
    pub fn free_all_kvmv_data();
    pub fn set_h264_gop(_gop: c_uchar);
    pub fn set_frame_detact(_frame_detact: c_uchar);
    pub fn kvmv_deinit();
    pub fn kvmv_hdmi_control(_en: c_uchar) -> c_uchar;
}
