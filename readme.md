# Rust-s3-uploader

### Blazingly fast s3 uploader in Rust

- Basically, the only bottleneck that we will encounter is the network.
- The app has incredible performance, being able to use less than 500MB of RAM (depending on the buffer size parameter)
and consumes an average of 10% of CPU (on a `r5.xlarge` on EC2, this one has an Intel Xeon 8175 Cpu, 4VCpus)
- It works by spawning tasks with `tokio::spawn`, inside the loop it reads the file, uploads it, and `unwrap()`

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