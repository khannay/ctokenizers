use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use tokenizers::Tokenizer;

// polars summarizer
use std::path::{Path, PathBuf};
use polars::prelude::*;
use glob::glob;
use std::{fs::File, io::Write};
//use libc::c_char;
use polars::lazy::dsl::col;

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
pub extern "C" fn analyze_network_dir(
    dir_path: *const c_char,
    top_n: i32
) -> i32 {
    // 1) CStr → &str → PathBuf
    let dir_str = unsafe {
        CStr::from_ptr(dir_path)
            .to_str()
            .map_err(|_| 1)
            .unwrap()
    };
    let dir = PathBuf::from(dir_str);

    // 2) Glob all CSVs
    let pattern = format!("{}/**/*.csv", dir.display());
    let mut frames = Vec::new();
    for entry in glob(&pattern).expect("Invalid glob pattern") {
        let path = match entry {
            Ok(p) => p,
            Err(_) => return 3,
        };

        // --- the “docs‐style” CSV reader: CsvReadOptions + try_into_reader_with_file_path
        let df = match CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some(path.clone())) // gives you a CsvReader<File>
        {
            Ok(mut rdr) => match rdr.finish() {
                Ok(df) => df,
                Err(_) => return 2,
            },
            Err(_) => return 2,
        };

        frames.push(df);
    }

    if frames.is_empty() {
        return 4;
    }

    // 3) Eagerly concatenate
    let mut combined = frames
        .into_iter()
        .reduce(|mut acc, df| { acc.vstack_mut(&df).unwrap(); acc })
        .unwrap();

    // 4) Lazy group/count/sort/limit
    let result = combined
        .lazy()
        .group_by(vec![
            col("source_ip"),
            col("source_port"),
            col("dest_ip"),
            col("dest_port"),
            col("protocol"),
            col("label"),
        ])
        .agg(vec![col("*").count().alias("count")])
        .sort_by_exprs(vec![col("count")],
		       SortMultipleOptions::default()
		       .with_order_descending(true)
		       .with_nulls_last(true),
	)
        .limit(top_n as u32)
        .collect()
        .map_err(|_| 5);

    let mut final_df = match result {
        Ok(df) => df,
        Err(code) => return code,
    };

    // 5) Parquet write
    let filename = dir
        .file_name()
        .map(|n| format!("{}_top.parquet", n.to_string_lossy()))
        .unwrap_or_else(|| "top.parquet".into());
    let out_path = dir.join(filename);

    let mut f = match File::create(&out_path) {
        Ok(f) => f,
        Err(_) => return 6,
    };
    if ParquetWriter::new(&mut f)
        .finish(&mut final_df)
        .is_err()
    {
        return 6;
    }

    0
}




