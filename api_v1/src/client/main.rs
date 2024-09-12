use grpc_client::{
    download, get_merkle_proof, setup_grpc_client, upload, verify_merkle_proofs, VerifyRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Declare the client as mutable
    let mut client = setup_grpc_client().await?;

    let files = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
        vec![10, 11, 12],
    ];

    // Pass the client as mutable reference
    let upload_response = upload(&mut client, files).await?;
    println!("Upload response: {:?}", upload_response);

    // Pass the client as mutable reference
    let download_response = download(&mut client, 3).await?;
    println!("Download response: {:?}", download_response);

    // Pass the client as mutable reference
    let proof_response_1 = get_merkle_proof(&mut client, 3).await?;
    println!("Proof response: {:?}", proof_response_1);

    // Independently verify the client proof - positive case
    let files_1 = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
        vec![10, 11, 12],
    ];
    let verify_request_1 = VerifyRequest {
        files: &files_1,

        root_hash: upload_response.root_hash,
        file_idx: 3,
        proofs: proof_response_1.proofs,
    };
    let verify_response_1 = verify_merkle_proofs(verify_request_1).await?;
    println!("Verify response: {:?}", verify_response_1);

    // Independently verify the client proof - negative case
    let files_2 = vec![vec![1, 2, 4], vec![4, 5, 6]];
    let proof_response_2 = get_merkle_proof(&mut client, 0).await?;
    let verify_request_2 = VerifyRequest {
        files: &files_2,

        root_hash: "d16e06cabb8ab6bacdedc91e3d786e7ad11d66525dd50635d882bf87a26abb75".to_string(),
        file_idx: 0,
        proofs: proof_response_2.proofs,
    };
    let verify_response_2 = verify_merkle_proofs(verify_request_2).await?;
    println!("Verify response: {:?}", verify_response_2);

    Ok(())
}
