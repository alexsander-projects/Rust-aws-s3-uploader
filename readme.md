# Rust-aws-s3-uploader

### Blazingly fast s3 uploader in Rust

- Basically, the only bottleneck that we will encounter is the network.
- The app has incredible performance, being able to use less than 500MB of RAM (depending on the buffer size parameter)
and consumes an average of 10% of CPU (on a `r5.xlarge` on EC2, this one has an Intel Xeon 8175 Cpu, 4VCpus)
- It works by spawning tasks with `tokio::spawn`, inside the loop it reads the file, uploads it, and `unwrap()`

## Here's the loop part of the code:

```doctestinjectablerust
let mut threads = vec![];
    for file_chunk in chunked_items {
        let shared_config = aws_config::from_env()
            .load()
            .await;
        let client = Client::new(&shared_config);

        let handle = tokio::spawn(async move {
            for file in file_chunk {
                println!("Uploading {}", &file.file_path);

                // Move file reading and upload logic inside the thread
                upload_multipart(&file, &client, chunk_size, buffer_size)
                    .await
                    .unwrap();
            }
        });
        threads.push(handle);
    }

    for thread in threads {
        thread.await.unwrap_or_else(|err| println!("{}", err));
    }
```

It loads the credentials with `aws_config::from_env()`
>Make sure to run aws configure on your cli, the region used there will be the region used by the app

- We may pass `buffer_size`, `chunk_size`, `bucket_name` and `threads` on the command that we will use to run the app
    
        ./s3_uploader.exe --bucket-name <bucket name> --dir-path <path-to-your-folder> --threads <number-of-threads> --chunk-size <chunk_size> --buffer-size <buffer_size>

- Buffer size:

    >Specify the size of the buffer used to read the file (in bytes).
Increasing the read buffer capacity to higher values than the default (4096 bytes) can result in a large
reduction in CPU usage, at the cost of memory increase.

- Chunk size:

    >Size of parts for the files, setting this to 500MB and then uploading a 1GB file would result in 2 parts to be
uploaded; A 1GB file would require 205 calls for the 205 chunks if using the current default of 5MB and so on...

As per documentation, here are the limits: [Amazon S3 multipart upload limits](https://docs.aws.amazon.com/AmazonS3/latest/userguide/qfacts.html)

- Bucket name:

    >The aws s3 bucket name. 

- Dir path:

    >The path to your folder/directory, when uploaded, folders will be created on s3 according to the path, for example: 
"C:/dir/data" will create the folders "dir" and "data" and put the files on those folders accordingly.

- Threads:

    >By documentation: https://v0-1--tokio.netlify.app/docs/futures/spawning/, spawn one thread per cpu core. These tasks will
be managed by the runtime, `tokio` is able to spawn many tasks per thread, improving a lot of performance, this is extremely
useful in our use case, since a lot of calls can be made.