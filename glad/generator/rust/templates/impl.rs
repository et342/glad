{% import 'template_utils.rs' as template_utils with context %}
pub use self::types::*;
pub use self::enumerations::*;
pub use self::functions::*;

use std::os::raw::c_void;
use super::{FnName, FnPtr};

{% set ctx_name = feature_set.name | capitalize %}

pub mod types {
    {% include 'types/' + spec.name + '.rs' ignore missing with context %}
}

pub mod enumerations {
    #![allow(dead_code, non_upper_case_globals, unused_imports)]

    use std::os::raw::*;
    use super::types::*;

    {% for enum in feature_set.enums %}
    pub const {{ enum.name|no_prefix }}: {{ enum|enum_type }} = {{ enum|enum_value }};
    {% endfor %}
}

pub mod functions {
    #![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

    use std::mem::transmute;
    use std::os::raw::*;
    use super::*;
    use super::types::*;

    macro_rules! func {
        ($fun:ident, $ret:ty, $($name:ident: $typ:ty),*) => {
            #[inline]
            #[track_caller]
            pub unsafe fn $fun({{ '&self, ' if options.mx }}$($name: $typ),*) -> $ret {
                if {{ 'self.' if options.mx else 'storage::' }}$fun.is_loaded() {
                    return transmute::<_, extern "system" fn($($typ),*) -> $ret>({{ 'self.' if options.mx else 'storage::' }}$fun.ptr)($($name),*);
                }
                panic!(concat!("{{ feature_set.name }}: function '", stringify!($fun), "' wasn't loaded"));
            }
        }
    }

    {% if options.mx %}
    pub struct {{ ctx_name }} {
        {% for command in feature_set.commands %}
        {{ template_utils.protect(command) }} pub(super) {{ command.name|no_prefix }}: FnPtr,
        {% endfor %}
    }

    {% if not spec.name | capitalize == ctx_name %}
    pub type {{ spec.name | capitalize }} = {{ ctx_name }};
    {% endif %}

    impl {{ ctx_name }} {
    {% endif %}

    {% for command in feature_set.commands %}
    {{ template_utils.protect(command) }} func!({{ command.name|no_prefix }}, {{ command.proto.ret|type }}, {{ command|params }});
    {% endfor %}

    {{ '}' if options.mx }}
}

{% if not options.mx %}
mod storage {
    #![allow(non_snake_case, non_upper_case_globals)]

    use super::FnPtr;

    macro_rules! store {
        ($name:ident) => {
            pub(super) static mut $name: FnPtr = FnPtr::new(std::ptr::null());
        }
    }

    {% for command in feature_set.commands %}
    {{ template_utils.protect(command) }} store!({{ command.name|no_prefix }});
    {% endfor %}
}
{% endif %}

{% if options.mx %}
pub fn load<F>(mut loadfn: F) -> functions::{{ ctx_name }} where F: FnMut(&'static FnName) -> *const c_void {
    #[allow(unused_mut)]
    let mut ctx = unsafe {
        {{ ctx_name }} {
            {% for command in feature_set.commands %}
            {{ template_utils.protect(command.name) }} {{ command.name|no_prefix }}: FnPtr::new(loadfn(FnName::from_bytes_with_nul_unchecked(b"{{ command.name }}\0"))),
            {% endfor %}
        }
    };

    {% for command, caliases in aliases|dictsort %}
    {% for alias in caliases|reject('equalto', command) %}
    {{ template_utils.protect(command) }} ctx.{{ command|no_prefix }}.aliased(&ctx.{{ alias|no_prefix }});
    {% endfor %}
    {% endfor %}

     ctx
}
{% else %}
pub fn load<F>(mut loadfn: F) where F: FnMut(&'static FnName) -> *const c_void {
    unsafe {
        {% for command in feature_set.commands %}
        {{ template_utils.protect(command) }} storage::{{ command.name | no_prefix }}.set_ptr(loadfn(FnName::from_bytes_with_nul_unchecked(b"{{ command.name }}\0")));
        {% endfor %}

        {% for command, caliases in aliases|dictsort %}
        {% for alias in caliases|reject('equalto', command) %}
        {{ template_utils.protect(command) }}{{ template_utils.protect(alias) }} storage::{{ command|no_prefix }}.aliased(&storage::{{ alias|no_prefix }});
        {% endfor %}
        {% endfor %}
    }
}
{% endif %}
