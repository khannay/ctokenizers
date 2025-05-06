use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use tokenizers::Tokenizer;

// polars summarizer
use std::path::{Path, PathBuf};
use polars::prelude::*;
use glob::glob;
use std::fs::File;

#[no_mangle]
pub extern "C" fn load_tokenizer(path: *const c_char) -> *mut Tokenizer {
    let c_path = unsafe { CStr::from_ptr(path) };
    let path_str = c_path.to_str().unwrap_or("");
    let tokenizer = match Tokenizer::from_file(path_str) {
        Ok(t) => Box::new(t),
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(tokenizer)
}

#[no_mangle]
pub extern "C" fn encode_text(tok: *mut Tokenizer, input: *const c_char, out_len: *mut usize) -> *mut u32 {
    let c_text = unsafe { CStr::from_ptr(input) };
    let text_str = c_text.to_str().unwrap_or("");

    let encoding = unsafe {
        match (*tok).encode(text_str, true) {
            Ok(enc) => enc,
            Err(_) => return ptr::null_mut(),
        }
    };

    let tokens = encoding.get_ids().to_vec();
    let len = tokens.len();
    let mut boxed = tokens.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    unsafe {
        *out_len = len;
    }
    ptr
}

#[no_mangle]
pub extern "C" fn free_encoded(ptr: *mut u32, len: usize) {
    unsafe {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
    }
}

#[no_mangle]
pub extern "C" fn free_tokenizer(ptr: *mut Tokenizer) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)); }
    }
}



#[no_mangle]
pub extern "C" fn encode_batch(
    tokenizer: *mut Tokenizer,
    inputs: *const *const c_char,
    num_inputs: usize,
    out_lengths: *mut usize,
) -> *mut *mut u32 {
    if tokenizer.is_null() || inputs.is_null() || out_lengths.is_null() {
        return ptr::null_mut();
    }

    let tokenizer = unsafe { &*tokenizer };

    let input_slices: Vec<String> = (0..num_inputs)
        .map(|i| {
            let c_str = unsafe { CStr::from_ptr(*inputs.add(i)) };
            c_str.to_string_lossy().into_owned()
        })
        .collect();

    let encodings = match tokenizer.encode_batch(input_slices, true) {
        Ok(e) => e,
        Err(_) => return ptr::null_mut(),
    };

    // Allocate pointer array
    let mut results: Vec<*mut u32> = Vec::with_capacity(num_inputs);

    for (i, encoding) in encodings.into_iter().enumerate() {
        let ids = encoding.get_ids().to_vec();
        let len = ids.len();

        let mut boxed = ids.into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        std::mem::forget(boxed); // leak it to return

        unsafe {
            *out_lengths.add(i) = len;
        }

        results.push(ptr);
    }

    let mut result_array = results.into_boxed_slice();
    let out_ptr = result_array.as_mut_ptr();
    std::mem::forget(result_array);
    out_ptr
}

#[no_mangle]
pub extern "C" fn free_encoded_batch(ptrs: *mut *mut u32, lengths: *const usize, count: usize) {
    if ptrs.is_null() || lengths.is_null() {
        return;
    }

    let ids_array = unsafe { std::slice::from_raw_parts_mut(ptrs, count) };
    let lens_array = unsafe { std::slice::from_raw_parts(lengths, count) };

    for i in 0..count {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(ids_array[i], lens_array[i]));
        }
    }

    unsafe {
        let _ = Box::from_raw(ids_array);
    }
}



#[no_mangle]
pub extern "C" fn analyze_network_dir(dir_path: *const c_char, top_n: i32) -> i32 {
    // Convert path
    let c_str = unsafe { CStr::from_ptr(dir_path) };
    let dir_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 1,
    };
    let dir = Path::new(dir_str);

    let pattern = dir.join("*.csv").to_string_lossy().to_string();
    let mut frames = vec![];

    // Read all CSVs
    for entry in glob(&pattern).expect("Bad pattern") {
        match entry {
            Ok(path) => {
                let file = match File::open(&path) {
                    Ok(f) => f,
                    Err(_) => return 1,
                };
                match CsvReader::new(file)
                    .has_header(true)
                    .finish()
                {
                    Ok(df) => frames.push(df),
                    Err(_) => return 2,
                }
            }
            Err(_) => return 3,
        }
    }
    

    if frames.is_empty() {
        return 4;
    }

    let concat = concat_df(&frames).unwrap();

    // Group and count
    let grouped = concat
        .lazy()
        .group_by([
            col("source_ip"),
            col("source_port"),
            col("dest_ip"),
            col("dest_port"),
            col("protocol"),
            col("label"),
        ])
        .agg([count().alias("count")])
        .sort("count", SortOptions {
            descending: true,
            ..Default::default()
        })
        .limit(top_n as u32)
        .collect();

    let final_df = match grouped {
        Ok(df) => df,
        Err(_) => return 5,
    };

    // Write to Parquet
    let out_path = dir
        .file_name()
        .map(|n| format!("{}_top.parquet", n.to_string_lossy()))
        .unwrap_or_else(|| "top.parquet".to_string());

    if let Err(_) = ParquetWriter::new(Path::new(&out_path)).finish(&final_df) {
        return 6;
    }

    0
}


