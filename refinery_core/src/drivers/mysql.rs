use crate::traits::sync::{Query, Transaction};
use crate::Migration;
use chrono::{DateTime, Local};
use mysql::{
    error::Error as MError, Conn, IsolationLevel, PooledConn, Transaction as MTransaction,
};

fn query_applied_migrations(
    transaction: &mut MTransaction,
    query: &str,
) -> Result<Vec<Migration>, MError> {
    let rows = transaction.query(query)?;
    let mut applied = Vec::new();
    for row in rows {
        let row = row?;
        let version = row.get(0).unwrap();
        let applied_on: String = row.get(2).unwrap();
        let applied_on = DateTime::parse_from_rfc3339(&applied_on)
            .unwrap()
            .with_timezone(&Local);
        let checksum: String = row.get(3).unwrap();

        applied.push(Migration::applied(
            version,
            row.get(1).unwrap(),
            applied_on,
            checksum.parse::<u64>().expect("checksum must be a valid u64")
        ))
    }
    Ok(applied)
}

impl Transaction for Conn {
    type Error = MError;

    fn execute(&mut self, queries: &[&str]) -> Result<usize, Self::Error> {
        let mut transaction =
            self.start_transaction(true, Some(IsolationLevel::RepeatableRead), None)?;
        let mut count = 0;
        for query in queries.iter() {
            transaction.query(query)?;
            count += 1;
        }
        transaction.commit()?;
        Ok(count as usize)
    }
}

impl Transaction for PooledConn {
    type Error = MError;

    fn execute(&mut self, queries: &[&str]) -> Result<usize, Self::Error> {
        let mut transaction =
            self.start_transaction(true, Some(IsolationLevel::RepeatableRead), None)?;
        let mut count = 0;

        for query in queries.iter() {
            transaction.query(query)?;
            count += 1;
        }
        transaction.commit()?;
        Ok(count as usize)
    }
}

impl Query<Vec<Migration>> for Conn {
    fn query(&mut self, query: &str) -> Result<Vec<Migration>, Self::Error> {
        let mut transaction =
            self.start_transaction(true, Some(IsolationLevel::RepeatableRead), None)?;
        let applied = query_applied_migrations(&mut transaction, query)?;
        transaction.commit()?;
        Ok(applied)
    }
}

impl Query<Vec<Migration>> for PooledConn {
    fn query(&mut self, query: &str) -> Result<Vec<Migration>, Self::Error> {
        let mut transaction =
            self.start_transaction(true, Some(IsolationLevel::RepeatableRead), None)?;
        let applied = query_applied_migrations(&mut transaction, query)?;
        transaction.commit()?;
        Ok(applied)
    }
}
