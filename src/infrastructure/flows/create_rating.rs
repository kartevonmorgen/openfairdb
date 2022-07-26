use super::*;

pub fn create_rating(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    rate_entry: usecases::NewPlaceRating,
) -> Result<(String, String)> {
    // Add new rating to existing entry
    let (rating_id, comment_id, place, status, ratings) = {
        let connection = connections.exclusive()?;
        connection.transaction(|| {
            match usecases::prepare_new_rating(&connection.inner(), rate_entry) {
                Ok(storable) => {
                    let rating_id = storable.rating_id().to_owned();
                    let comment_id = storable.comment_id().to_owned();
                    let (place, status, ratings) =
                        usecases::store_new_rating(&connection.inner(), storable).map_err(
                            |err| {
                                warn!("Failed to store new rating for entry: {}", err);
                                TransactionError::RollbackTransaction
                            },
                        )?;
                    Ok((rating_id, comment_id, place, status, ratings))
                }
                Err(err) => Err(TransactionError::Usecase(err)),
            }
        })
    }?;

    // Reindex entry after adding the new rating
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = usecases::reindex_place(indexer, &place, status, &ratings)
        .and_then(|_| indexer.flush_index())
    {
        error!(
            "Failed to reindex place {} after adding a new rating: {}",
            place.id, err
        );
    }

    Ok((rating_id, comment_id))
}
