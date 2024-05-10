use clap::Parser;
use itertools::Itertools;
use walkdir::WalkDir;
use tokio;
use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::Client; // Import the S3 client
use aws_smithy_types::byte_stream::{ByteStream, Length}; // Import the ByteStream struct


#[derive(Parser, Debug, Clone)] // Derive the Parser trait for the Args struct
#[command(author, version, about, long_about = None)] // Define the command attributes
struct Args {
    #[arg(short, long)]
    bucket_name: String, // argument for the bucket name

    #[arg(short, long)] // argument for the directory path
    dir_path: String,

    #[arg(short, long)]
    threads: usize,

    #[arg(short, long)] // 5 MB in bytes, default_value_t = 5242880
    chunk_size: u64,

    #[arg(short = 'm', long)] // 512 KB in bytes, default_value_t = 524288
    buffer_size: usize,

    #[arg(short = 'p', long)] // argument for the S3 path
    s3_path: String,
}

struct FileUpload {
    file_path: String,
    bucket_name: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let thread_count = args.threads;
    let chunk_size = args.chunk_size;
    let buffer_size = args.buffer_size;
    let s3_path = args.s3_path;

    // Collect only file paths upfront
    let mut file_paths = Vec::new();
    for file_path in WalkDir::new(args.dir_path) {
        let file_path = file_path.unwrap();
        if file_path.path().is_file() {
            file_paths.push(FileUpload {
                file_path: file_path.path().to_str().unwrap().to_owned(),
                bucket_name: args.bucket_name.to_owned()
            });
        }
    }

    // Divide work into chunks
    let chunked_items: Vec<Vec<FileUpload>> = file_paths
        .into_iter()
        .chunks(thread_count)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();


    // Start threads
    let mut threads = vec![];
    for file_chunk in chunked_items {
        let shared_config = aws_config::from_env()
            .load()
            .await;
        let client = Client::new(&shared_config);

        let s3_path = s3_path.clone();

        let handle = tokio::spawn(async move {
            for file in file_chunk {
                println!("Uploading {}", &file.file_path);
                upload_multipart(&file, &client, chunk_size, buffer_size, &s3_path)
                    .await
                    .unwrap();
            }
        });
        threads.push(handle);
    }

    for thread in threads {
        thread.await.unwrap_or_else(|err| println!("{}", err));
    }
}

async fn upload_multipart(file_upload: &FileUpload, client: &Client, chunk_size: u64, buffer_size: usize, s3_path: &str) -> anyhow::Result<()> {
    let file_path = file_upload.file_path.to_owned();
    let bucket_name = &file_upload.bucket_name;
    let file_size = tokio::fs::metadata(&file_upload.file_path)
        .await
        .expect("it exists I swear")
        .len();

    // Extract the parent directory and file name from the file path
    let path = std::path::Path::new(&file_path);
    let file_name = path.file_name().unwrap().to_str().unwrap();

    // Create the key for the S3 bucket
    let key = format!("{}/{}", s3_path, file_name);

    let multipart_upload_res: CreateMultipartUploadOutput = client
        .create_multipart_upload()
        .bucket(bucket_name)
        .key(key.clone())
        .send()
        .await
        .unwrap();

    let upload_id = multipart_upload_res.upload_id().unwrap();

    println!("Multipart Upload ID: {}", upload_id);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let mut chunk_count = (file_size / chunk_size) + 1;
    let mut size_of_last_chunk = file_size % chunk_size;
    if size_of_last_chunk == 0 {
        size_of_last_chunk = chunk_size;
        chunk_count -= 1;
    }

    if file_size == 0 {
        return Ok(());
    }

    let mut upload_parts: Vec<CompletedPart> = Vec::new();

    for chunk_index in 0..chunk_count {

        println!("Uploading chunk {} of {}", chunk_index + 1, chunk_count);
        let this_chunk = if chunk_count - 1 == chunk_index {
            size_of_last_chunk
        } else {
            chunk_size
        };

        let stream = ByteStream::read_from() // Create a ByteStream from the file path
            .path(file_path.clone())
            .offset(chunk_index * chunk_size)
            .length(Length::Exact(this_chunk))
            .buffer_size(buffer_size)
            .build()
            .await
            .unwrap();
        println!("Buffer size: {}", buffer_size);
        println!("Path: {}", file_path);

        //Chunk index needs to start at 0, but part numbers start at 1.
        let part_number = (chunk_index as i32) + 1;

        let upload_part_res = client
            .upload_part()
            .key(key.clone())
            .bucket(bucket_name)
            .upload_id(upload_id)
            .body(stream)
            .part_number(part_number)
            .send()
            .await?;

        upload_parts.push(
            CompletedPart::builder()
                .e_tag(upload_part_res.e_tag.unwrap_or_default())
                .part_number(part_number)
                .build(),
        );
    }

    let completed_multipart_upload: CompletedMultipartUpload = CompletedMultipartUpload::builder()
        .set_parts(Some(upload_parts))
        .build();

    let _complete_multipart_upload_res = client
        .complete_multipart_upload()
        .bucket(bucket_name)
        .key(key)
        .multipart_upload(completed_multipart_upload)
        .upload_id(upload_id)
        .send()
        .await
        .unwrap();

    println!("Upload complete for file: {}", file_path);
    println!("upload_id: {}", upload_id);
    println!("bucket_name: {}", bucket_name);

    Ok(())
}