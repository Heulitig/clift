pub enum Uploader {
    File(tokio::fs::File),
    S3(clift::api::PreSignedRequest, Vec<u8>),
}

#[derive(thiserror::Error, Debug)]
pub enum UploaderError {
    #[error("io error {0}")]
    IOError(#[from] std::io::Error),
    #[error("reqwest error {0}")]
    S3(#[from] reqwest::Error),
}

impl Uploader {
    pub async fn debug(path: &std::path::Path) -> Result<Uploader, UploaderError> {
        let file = tokio::fs::File::create(path).await?;
        Ok(Uploader::File(file))
    }

    pub fn s3(sr: clift::api::PreSignedRequest) -> Uploader {
        Uploader::S3(sr, vec![])
    }

    pub async fn upload(&mut self, path: &std::path::Path) -> Result<(), UploaderError> {
        use tokio::io::AsyncWriteExt;
        match self {
            Uploader::File(file) => file.write_all(&tokio::fs::read(path).await?).await?,
            Uploader::S3(_, v) => {
                v.append(&mut tokio::fs::read(path).await?);
            }
        }
        Ok(())
    }

    pub async fn commit(&mut self) -> Result<(), UploaderError> {
        if let Uploader::S3(sr, v) = self {
            let client = reqwest::Client::new();
            let mut request = client.request(
                reqwest::Method::from_bytes(sr.method.as_bytes()).unwrap(),
                &sr.url,
            );
            for (k, v) in sr.headers.iter() {
                request = request.header(k, v);
            }

            request.body(v.clone()).send().await?;
        }

        Ok(())
    }
}
