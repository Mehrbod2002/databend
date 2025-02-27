// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::hash::Hash;

use common_expression::types::nullable::NullableDomain;
use common_expression::types::ArrayType;
use common_expression::types::EmptyArrayType;
use common_expression::types::EmptyMapType;
use common_expression::types::GenericType;
use common_expression::types::MapType;
use common_expression::types::NullType;
use common_expression::types::NullableType;
use common_expression::vectorize_with_builder_2_arg;
use common_expression::FunctionDomain;
use common_expression::FunctionRegistry;
use common_expression::Value;
use common_hashtable::StackHashSet;
use siphasher::sip128::Hasher128;
use siphasher::sip128::SipHasher24;

pub fn register(registry: &mut FunctionRegistry) {
    registry
        .register_passthrough_nullable_2_arg::<EmptyArrayType, EmptyArrayType, EmptyMapType, _, _>(
            "map",
            |_, _| FunctionDomain::Full,
            |_, _, _| Value::Scalar(()),
        );

    registry.register_passthrough_nullable_2_arg::<ArrayType<GenericType<0>>, ArrayType<GenericType<1>>, MapType<GenericType<0>, GenericType<1>>, _, _>(
        "map",
        |_, _| FunctionDomain::MayThrow,
        vectorize_with_builder_2_arg::<ArrayType<GenericType<0>>, ArrayType<GenericType<1>>, MapType<GenericType<0>, GenericType<1>>>(
            |keys, vals, output, ctx| {
                let key_type = &ctx.generics[0];
                if !key_type.is_boolean()
                        && !key_type.is_string()
                        && !key_type.is_numeric()
                        && !key_type.is_decimal()
                        && !key_type.is_date_or_date_time() {
                    ctx.set_error(output.len(), format!("map keys can not be {}", key_type));
                } else if keys.len() != vals.len() {
                    ctx.set_error(output.len(), format!(
                        "key list has a different size from value list ({} keys, {} values)",
                        keys.len(), vals.len()
                    ));
                } else if keys.len() <= 1 {
                    for idx in 0..keys.len() {
                        let key = unsafe { keys.index_unchecked(idx) };
                        let val = unsafe { vals.index_unchecked(idx) };
                        output.put_item((key, val));
                    }
                } else {
                    let mut set: StackHashSet<u128, 16> =
                        StackHashSet::with_capacity(keys.len());
                    for idx in 0..keys.len() {
                        let key = unsafe { keys.index_unchecked(idx) };
                        let mut hasher = SipHasher24::new();
                        key.hash(&mut hasher);
                        let hash128 = hasher.finish128();
                        let hash_key = hash128.into();
                        if set.contains(&hash_key) {
                            ctx.set_error(output.len(), "map keys have to be unique");
                            break;
                        }
                        let _ = set.set_insert(hash_key);
                        let val = unsafe { vals.index_unchecked(idx) };
                        output.put_item((key, val));
                    }
                }
                output.commit_row();
            }
        ),
    );

    registry.register_2_arg_core::<NullableType<EmptyMapType>, NullableType<GenericType<0>>, NullType, _, _>(
        "get",
        |_, _| FunctionDomain::Full,
        |_, _, _| Value::Scalar(()),
    );

    registry.register_combine_nullable_2_arg::<MapType<GenericType<0>, GenericType<1>>, GenericType<0>, GenericType<1>, _, _>(
        "get",
        |domain, _| FunctionDomain::Domain(NullableDomain {
            has_null: true,
            value: domain.as_ref().map(|(_, val_domain)| Box::new(val_domain.clone())),
        }),
        vectorize_with_builder_2_arg::<MapType<GenericType<0>, GenericType<1>>, GenericType<0>, NullableType<GenericType<1>>>(
            |map, key, output, _| {
                for (k, v) in map.iter() {
                    if k == key {
                        output.push(v);
                        return
                    }
                }
                output.push_null()
            }
        ),
    );
}
