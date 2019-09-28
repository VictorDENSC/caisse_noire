use diesel::prelude::*;
use std::ops::Deref;
use uuid::Uuid;

use super::{interface::UsersDb, models::User};
use crate::database::{
    postgres::{DbConnection, DbError},
    schema::users,
};

impl UsersDb for DbConnection {
    fn get_users(&self, team_id: Uuid) -> Result<Vec<User>, DbError> {
        let users: Vec<User> = users::table
            .filter(users::team_id.eq(team_id))
            .get_results(self.deref())?;

        Ok(users)
    }

    fn get_user_by_id(&self, team_id: Uuid, user_id: Uuid) -> Result<User, DbError> {
        let user: User = users::table
            .filter(users::team_id.eq(team_id).and(users::id.eq(user_id)))
            .get_result(self.deref())?;

        Ok(user)
    }

    fn create_user(&self, user: &User) -> Result<User, DbError> {
        let user: User = diesel::insert_into(users::table)
            .values(user)
            .get_result(self.deref())?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use diesel::result::Error;

    use super::*;
    use crate::database::postgres::test_utils::{
        create_default_team, create_default_user, DbConnectionBuilder,
    };

    #[test]
    fn test_get_users() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let new_user = create_default_user(&conn, "login", "password");
            create_default_user(&conn, "login_2", "password_2");

            let users = conn.get_users(new_user.team_id).unwrap();

            assert_eq!(vec![new_user], users);

            Ok(())
        })
    }

    #[test]
    fn test_get_user() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let new_user = create_default_user(&conn, "login", "password");

            let user = conn.get_user_by_id(new_user.team_id, new_user.id).unwrap();

            assert_eq!(new_user, user);

            Ok(())
        })
    }

    #[test]
    fn test_get_unexisting_user() {
        let conn = DbConnectionBuilder::new();

        let error = conn
            .get_user_by_id(Uuid::new_v4(), Uuid::new_v4())
            .unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }

    #[test]
    fn test_create_user() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let team = create_default_team(&conn);

            let new_user = User {
                id: Uuid::new_v4(),
                team_id: team.id,
                firstname: String::from("firstname"),
                lastname: String::from("lastname"),
                nickname: None,
                login: String::from("login"),
                password: String::from("password"),
                email: None,
            };

            let user = conn.create_user(&new_user).unwrap();

            assert_eq!(new_user, user);

            Ok(())
        })
    }

    #[test]
    fn test_create_uncorrect_user() {
        let conn = DbConnectionBuilder::new();
    
        conn.deref().test_transaction::<_, Error, _>(|| {
            let mut new_user = User {
                id: Uuid::new_v4(),
                team_id: Uuid::new_v4(),
                firstname: String::from("firstname"),
                lastname: String::from("lastname"),
                nickname: None,
                login: String::from("login"),
                password: String::from("password"),
                email: None,
            };

            let error = conn.create_user(&new_user).unwrap_err();

            assert_eq!(error, DbError::Unknown);

            Ok(())
        })
    }
}
