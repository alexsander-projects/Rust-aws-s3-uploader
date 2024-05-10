# Rust-aws-s3-uploader

## Blazingly fast s3 uploader in Rust

- The app is able to upload files of any size, it will split the file into parts and upload them in parallel, the number of
  parts is determined by the `chunk_size` parameter, the default is 5MB, but it can be changed.
- Basically, the only bottleneck that we will encounter is the network or the disk speed.
- The app has incredible performance, being able to use less than 500MB of RAM (depending on the buffer size parameter)
and consumes an average of 1% of CPU (on a `r5.xlarge` on EC2, this one has an Intel Xeon 8175 Cpu, 4VCpus)
- Note that the process itself is not CPU-bound, it is network-bound, so the CPU usage is low, but the cpu on the above scenario
can be at an average of 15% due to high usage of network and disk.

### How to run

- Run `cargo build --release`
- Run the app with the parameters that you want, for example:

    `./s3_uploader.exe --bucket-name <bucket name> --dir-path <path-to-your-folder> --threads <number-of-threads> --chunk-size <chunk_size> --buffer-size <buffer_size> --s3-path <s3-bucket-path>`
>`--s3-path` is optional, if not provided, the files will be uploaded to the root of the bucket, if provided, the files will be uploaded to the path provided.
- The app will read the files from the directory and upload them to the s3 bucket.

## How it works

- It works by spawning a thread per core, and then using the tokio runtime to manage the tasks, this way we can have a lot of
  tasks running in parallel, improving the performance. Spawning a task with Tokio is extremely lightweight.

The app loads the credentials from the environment variables, so make sure to have the `AWS_ACCESS_KEY_ID` and `AWS_SECRET
_ACCESS_KEY` set on your environment. Remember to have the correct permissions on the bucket.

### Parameters

- Buffer size:

    >Specify the size of the buffer used to read the file (in bytes).
Increasing the read buffer capacity to higher values than the default (4096 bytes) can result in a large
reduction in CPU usage, at the cost of RAM usage.

- Chunk size:

    >Size of parts for the files, setting this to 500MB and then uploading a 1GB file would result in 2 parts to be
uploaded; A 1GB file would require 205 calls for the 205 chunks if using the current default of 5MB and so on...

As pre documentation, here are the limits:  [Amazon S3 multipart upload limits](https://docs.aws.amazon.com/AmazonS3/latest/userguide/qfacts.html)

- Bucket name:

    >The name of the bucket that you want to upload the files to.

- Dir path:

    >The path to the directory that contains the files that you want to upload. The app will read all the files in the
directory and upload them to the bucket.

- Threads:

    >By documentation: https://v0-1--tokio.netlify.app/docs/futures/spawning/, spawn one thread per cpu core. These tasks will
be managed by the runtime, `tokio` is able to spawn many tasks per thread, improving the performance.