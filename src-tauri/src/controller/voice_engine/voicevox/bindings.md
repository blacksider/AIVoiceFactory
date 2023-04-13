```rust
/* automatically generated by rust-bindgen 0.64.0 */

pub const __bool_true_false_are_defined: u32 = 1;
pub const true_: u32 = 1;
pub const false_: u32 = 0;
pub const _VCRT_COMPILER_PREPROCESSOR: u32 = 1;
pub const _SAL_VERSION: u32 = 20;
pub const __SAL_H_VERSION: u32 = 180000000;
pub const _USE_DECLSPECS_FOR_SAL: u32 = 0;
pub const _USE_ATTRIBUTES_FOR_SAL: u32 = 0;
pub const _CRT_PACKING: u32 = 8;
pub const _HAS_EXCEPTIONS: u32 = 1;
pub const _STL_LANG: u32 = 0;
pub const _HAS_CXX17: u32 = 0;
pub const _HAS_CXX20: u32 = 0;
pub const _HAS_CXX23: u32 = 0;
pub const _HAS_NODISCARD: u32 = 0;
pub const WCHAR_MIN: u32 = 0;
pub const WCHAR_MAX: u32 = 65535;
pub const WINT_MIN: u32 = 0;
pub const WINT_MAX: u32 = 65535;
pub type va_list = *mut ::std::os::raw::c_char;
extern "C" {
    pub fn __va_start(arg1: *mut *mut ::std::os::raw::c_char, ...);
}
pub type __vcrt_bool = bool;
pub type wchar_t = ::std::os::raw::c_ushort;
extern "C" {
    pub fn __security_init_cookie();
}
extern "C" {
    pub fn __security_check_cookie(_StackCookie: usize);
}
extern "C" {
    pub fn __report_gsfailure(_StackCookie: usize) -> !;
}
extern "C" {
    pub static mut __security_cookie: usize;
}
pub type int_least8_t = ::std::os::raw::c_schar;
pub type int_least16_t = ::std::os::raw::c_short;
pub type int_least32_t = ::std::os::raw::c_int;
pub type int_least64_t = ::std::os::raw::c_longlong;
pub type uint_least8_t = ::std::os::raw::c_uchar;
pub type uint_least16_t = ::std::os::raw::c_ushort;
pub type uint_least32_t = ::std::os::raw::c_uint;
pub type uint_least64_t = ::std::os::raw::c_ulonglong;
pub type int_fast8_t = ::std::os::raw::c_schar;
pub type int_fast16_t = ::std::os::raw::c_int;
pub type int_fast32_t = ::std::os::raw::c_int;
pub type int_fast64_t = ::std::os::raw::c_longlong;
pub type uint_fast8_t = ::std::os::raw::c_uchar;
pub type uint_fast16_t = ::std::os::raw::c_uint;
pub type uint_fast32_t = ::std::os::raw::c_uint;
pub type uint_fast64_t = ::std::os::raw::c_ulonglong;
pub type intmax_t = ::std::os::raw::c_longlong;
pub type uintmax_t = ::std::os::raw::c_ulonglong;
#[doc = " 実行環境に合った適切なハードウェアアクセラレーションモードを選択する"]
pub const VoicevoxAccelerationMode_VOICEVOX_ACCELERATION_MODE_AUTO: VoicevoxAccelerationMode = 0;
#[doc = " ハードウェアアクセラレーションモードを\"CPU\"に設定する"]
pub const VoicevoxAccelerationMode_VOICEVOX_ACCELERATION_MODE_CPU: VoicevoxAccelerationMode = 1;
#[doc = " ハードウェアアクセラレーションモードを\"GPU\"に設定する"]
pub const VoicevoxAccelerationMode_VOICEVOX_ACCELERATION_MODE_GPU: VoicevoxAccelerationMode = 2;
pub type VoicevoxAccelerationMode = i32;
#[doc = " 成功"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_OK: VoicevoxResultCode = 0;
#[doc = " open_jtalk辞書ファイルが読み込まれていない"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR: VoicevoxResultCode =
    1;
#[doc = " modelの読み込みに失敗した"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_LOAD_MODEL_ERROR: VoicevoxResultCode = 2;
#[doc = " サポートされているデバイス情報取得に失敗した"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR: VoicevoxResultCode = 3;
#[doc = " GPUモードがサポートされていない"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_GPU_SUPPORT_ERROR: VoicevoxResultCode = 4;
#[doc = " メタ情報読み込みに失敗した"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_LOAD_METAS_ERROR: VoicevoxResultCode = 5;
#[doc = " ステータスが初期化されていない"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_UNINITIALIZED_STATUS_ERROR: VoicevoxResultCode = 6;
#[doc = " 無効なspeaker_idが指定された"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_INVALID_SPEAKER_ID_ERROR: VoicevoxResultCode = 7;
#[doc = " 無効なmodel_indexが指定された"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR: VoicevoxResultCode = 8;
#[doc = " 推論に失敗した"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_INFERENCE_ERROR: VoicevoxResultCode = 9;
#[doc = " コンテキストラベル出力に失敗した"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR: VoicevoxResultCode =
    10;
#[doc = " 無効なutf8文字列が入力された"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR: VoicevoxResultCode = 11;
#[doc = " aquestalk形式のテキストの解析に失敗した"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_PARSE_KANA_ERROR: VoicevoxResultCode = 12;
#[doc = " 無効なAudioQuery"]
pub const VoicevoxResultCode_VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR: VoicevoxResultCode = 13;
pub type VoicevoxResultCode = i32;
#[doc = " 初期化オプション"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoicevoxInitializeOptions {
    #[doc = " ハードウェアアクセラレーションモード"]
    pub acceleration_mode: VoicevoxAccelerationMode,
    #[doc = " CPU利用数を指定\n 0を指定すると環境に合わせたCPUが利用される"]
    pub cpu_num_threads: u16,
    #[doc = " 全てのモデルを読み込む"]
    pub load_all_models: bool,
    #[doc = " open_jtalkの辞書ディレクトリ"]
    pub open_jtalk_dict_dir: *const ::std::os::raw::c_char,
}
#[test]
fn bindgen_test_layout_VoicevoxInitializeOptions() {
    const UNINIT: ::std::mem::MaybeUninit<VoicevoxInitializeOptions> =
        ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<VoicevoxInitializeOptions>(),
        16usize,
        concat!("Size of: ", stringify!(VoicevoxInitializeOptions))
    );
    assert_eq!(
        ::std::mem::align_of::<VoicevoxInitializeOptions>(),
        8usize,
        concat!("Alignment of ", stringify!(VoicevoxInitializeOptions))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).acceleration_mode) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(VoicevoxInitializeOptions),
            "::",
            stringify!(acceleration_mode)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).cpu_num_threads) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(VoicevoxInitializeOptions),
            "::",
            stringify!(cpu_num_threads)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).load_all_models) as usize - ptr as usize },
        6usize,
        concat!(
            "Offset of field: ",
            stringify!(VoicevoxInitializeOptions),
            "::",
            stringify!(load_all_models)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).open_jtalk_dict_dir) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(VoicevoxInitializeOptions),
            "::",
            stringify!(open_jtalk_dict_dir)
        )
    );
}
#[doc = " Audio query のオプション"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoicevoxAudioQueryOptions {
    #[doc = " aquestalk形式のkanaとしてテキストを解釈する"]
    pub kana: bool,
}
#[test]
fn bindgen_test_layout_VoicevoxAudioQueryOptions() {
    const UNINIT: ::std::mem::MaybeUninit<VoicevoxAudioQueryOptions> =
        ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<VoicevoxAudioQueryOptions>(),
        1usize,
        concat!("Size of: ", stringify!(VoicevoxAudioQueryOptions))
    );
    assert_eq!(
        ::std::mem::align_of::<VoicevoxAudioQueryOptions>(),
        1usize,
        concat!("Alignment of ", stringify!(VoicevoxAudioQueryOptions))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).kana) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(VoicevoxAudioQueryOptions),
            "::",
            stringify!(kana)
        )
    );
}
#[doc = " `voicevox_synthesis` のオプション"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoicevoxSynthesisOptions {
    #[doc = " 疑問文の調整を有効にする"]
    pub enable_interrogative_upspeak: bool,
}
#[test]
fn bindgen_test_layout_VoicevoxSynthesisOptions() {
    const UNINIT: ::std::mem::MaybeUninit<VoicevoxSynthesisOptions> =
        ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<VoicevoxSynthesisOptions>(),
        1usize,
        concat!("Size of: ", stringify!(VoicevoxSynthesisOptions))
    );
    assert_eq!(
        ::std::mem::align_of::<VoicevoxSynthesisOptions>(),
        1usize,
        concat!("Alignment of ", stringify!(VoicevoxSynthesisOptions))
    );
    assert_eq!(
        unsafe {
            ::std::ptr::addr_of!((*ptr).enable_interrogative_upspeak) as usize - ptr as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(VoicevoxSynthesisOptions),
            "::",
            stringify!(enable_interrogative_upspeak)
        )
    );
}
#[doc = " テキスト音声合成オプション"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoicevoxTtsOptions {
    #[doc = " aquestalk形式のkanaとしてテキストを解釈する"]
    pub kana: bool,
    #[doc = " 疑問文の調整を有効にする"]
    pub enable_interrogative_upspeak: bool,
}
#[test]
fn bindgen_test_layout_VoicevoxTtsOptions() {
    const UNINIT: ::std::mem::MaybeUninit<VoicevoxTtsOptions> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<VoicevoxTtsOptions>(),
        2usize,
        concat!("Size of: ", stringify!(VoicevoxTtsOptions))
    );
    assert_eq!(
        ::std::mem::align_of::<VoicevoxTtsOptions>(),
        1usize,
        concat!("Alignment of ", stringify!(VoicevoxTtsOptions))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).kana) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(VoicevoxTtsOptions),
            "::",
            stringify!(kana)
        )
    );
    assert_eq!(
        unsafe {
            ::std::ptr::addr_of!((*ptr).enable_interrogative_upspeak) as usize - ptr as usize
        },
        1usize,
        concat!(
            "Offset of field: ",
            stringify!(VoicevoxTtsOptions),
            "::",
            stringify!(enable_interrogative_upspeak)
        )
    );
}
extern "C" {
    pub fn voicevox_make_default_initialize_options() -> VoicevoxInitializeOptions;
}
extern "C" {
    pub fn voicevox_initialize(options: VoicevoxInitializeOptions) -> VoicevoxResultCode;
}
extern "C" {
    pub fn voicevox_get_version() -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn voicevox_load_model(speaker_id: u32) -> VoicevoxResultCode;
}
extern "C" {
    pub fn voicevox_is_gpu_mode() -> bool;
}
extern "C" {
    pub fn voicevox_is_model_loaded(speaker_id: u32) -> bool;
}
extern "C" {
    pub fn voicevox_finalize();
}
extern "C" {
    pub fn voicevox_get_metas_json() -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn voicevox_get_supported_devices_json() -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn voicevox_predict_duration(
        length: usize,
        phoneme_vector: *mut i64,
        speaker_id: u32,
        output_predict_duration_data_length: *mut usize,
        output_predict_duration_data: *mut *mut f32,
    ) -> VoicevoxResultCode;
}
extern "C" {
    pub fn voicevox_predict_duration_data_free(predict_duration_data: *mut f32);
}
extern "C" {
    pub fn voicevox_predict_intonation(
        length: usize,
        vowel_phoneme_vector: *mut i64,
        consonant_phoneme_vector: *mut i64,
        start_accent_vector: *mut i64,
        end_accent_vector: *mut i64,
        start_accent_phrase_vector: *mut i64,
        end_accent_phrase_vector: *mut i64,
        speaker_id: u32,
        output_predict_intonation_data_length: *mut usize,
        output_predict_intonation_data: *mut *mut f32,
    ) -> VoicevoxResultCode;
}
extern "C" {
    pub fn voicevox_predict_intonation_data_free(predict_intonation_data: *mut f32);
}
extern "C" {
    pub fn voicevox_decode(
        length: usize,
        phoneme_size: usize,
        f0: *mut f32,
        phoneme_vector: *mut f32,
        speaker_id: u32,
        output_decode_data_length: *mut usize,
        output_decode_data: *mut *mut f32,
    ) -> VoicevoxResultCode;
}
extern "C" {
    pub fn voicevox_decode_data_free(decode_data: *mut f32);
}
extern "C" {
    pub fn voicevox_make_default_audio_query_options() -> VoicevoxAudioQueryOptions;
}
extern "C" {
    pub fn voicevox_audio_query(
        text: *const ::std::os::raw::c_char,
        speaker_id: u32,
        options: VoicevoxAudioQueryOptions,
        output_audio_query_json: *mut *mut ::std::os::raw::c_char,
    ) -> VoicevoxResultCode;
}
extern "C" {
    pub fn voicevox_make_default_synthesis_options() -> VoicevoxSynthesisOptions;
}
extern "C" {
    pub fn voicevox_synthesis(
        audio_query_json: *const ::std::os::raw::c_char,
        speaker_id: u32,
        options: VoicevoxSynthesisOptions,
        output_wav_length: *mut usize,
        output_wav: *mut *mut u8,
    ) -> VoicevoxResultCode;
}
extern "C" {
    pub fn voicevox_make_default_tts_options() -> VoicevoxTtsOptions;
}
extern "C" {
    pub fn voicevox_tts(
        text: *const ::std::os::raw::c_char,
        speaker_id: u32,
        options: VoicevoxTtsOptions,
        output_wav_length: *mut usize,
        output_wav: *mut *mut u8,
    ) -> VoicevoxResultCode;
}
extern "C" {
    pub fn voicevox_audio_query_json_free(audio_query_json: *mut ::std::os::raw::c_char);
}
extern "C" {
    pub fn voicevox_wav_free(wav: *mut u8);
}
extern "C" {
    pub fn voicevox_error_result_to_message(
        result_code: VoicevoxResultCode,
    ) -> *const ::std::os::raw::c_char;
}
```