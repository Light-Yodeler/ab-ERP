use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::Sha512;

const PBKDF2_ITERATIONS: u32 = 10_000;
const KEY_LEN: usize = 64; // 512 bits

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub username: String,
    pub full_name: String,
    pub role: String,
    pub created_at: Option<String>,
    pub last_login: Option<String>,
}

/// Generates a 16-byte random salt as a 32-character lowercase hex string.
pub fn generate_salt() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Hashes the given PIN with the salt string using PBKDF2-HMAC-SHA512 (10,000 iterations).
/// Note: Salt is passed as utf-8 bytes (identical to legacy Node.js crypto implementation).
pub fn hash_pin(pin: &str, salt: &str) -> String {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha512>(pin.as_bytes(), salt.as_bytes(), PBKDF2_ITERATIONS, &mut key);
    hex::encode(key)
}

/// Verifies whether the provided PIN matches the expected hash using constant-time comparison.
pub fn verify_pin(pin: &str, salt: &str, expected_hash: &str) -> bool {
    let computed = hash_pin(pin, salt);
    // Constant-time comparison
    if computed.len() != expected_hash.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in computed.bytes().zip(expected_hash.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Seeds default accounts ('admin' with PIN 123456, 'staff' with PIN 1234) if users table is empty.
pub fn seed_default_users(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    if count == 0 {
        // Admin
        let salt_admin = generate_salt();
        let hash_admin = hash_pin("123456", &salt_admin);
        conn.execute(
            "INSERT INTO users (username, pin_hash, salt, full_name, role) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["admin", hash_admin, salt_admin, "Administrator", "admin"],
        )?;

        // Staff
        let salt_staff = generate_salt();
        let hash_staff = hash_pin("1234", &salt_staff);
        conn.execute(
            "INSERT INTO users (username, pin_hash, salt, full_name, role) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["staff", hash_staff, salt_staff, "Staff Pelaporan", "staff"],
        )?;
    }
    Ok(())
}

/// Authenticates a user by username and PIN. Returns UserProfile on success.
pub fn authenticate(
    conn: &Connection,
    username: &str,
    pin: &str,
) -> Result<UserProfile, String> {
    let user_opt = conn
        .query_row(
            "SELECT id, username, pin_hash, salt, full_name, role, created_at, last_login FROM users WHERE username = ?1",
            params![username],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => "Pengguna tidak ditemukan.".to_string(),
            _ => format!("Kesalahan database: {}", e),
        })?;

    let (id, uname, pin_hash, salt, full_name, role, created_at, _) = user_opt;

    if !verify_pin(pin, &salt, &pin_hash) {
        return Err("PIN yang Anda masukkan salah.".to_string());
    }

    // Update last_login timestamp with local system time
    let now_local = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = conn.execute(
        "UPDATE users SET last_login = ?1 WHERE id = ?2",
        params![now_local, id],
    );

    Ok(UserProfile {
        id,
        username: uname,
        full_name,
        role,
        created_at,
        last_login: Some(now_local),
    })
}

/// Lists all users (without exposing pin_hash and salt).
pub fn list_users(conn: &Connection) -> Result<Vec<UserProfile>, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, full_name, role, created_at, last_login FROM users ORDER BY id ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(UserProfile {
                id: row.get(0)?,
                username: row.get(1)?,
                full_name: row.get(2)?,
                role: row.get(3)?,
                created_at: row.get(4)?,
                last_login: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut users = Vec::new();
    for r in rows {
        users.push(r.map_err(|e| e.to_string())?);
    }
    Ok(users)
}

/// Creates a new user (admin only).
pub fn create_user(
    conn: &Connection,
    username: &str,
    pin: &str,
    full_name: &str,
    role: &str,
) -> Result<UserProfile, String> {
    if role != "admin" && role != "staff" {
        return Err("Peran tidak valid. Harus 'admin' atau 'staff'.".to_string());
    }
    if pin.len() < 4 || pin.len() > 8 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN harus berupa 4 hingga 8 digit angka.".to_string());
    }

    let salt = generate_salt();
    let pin_hash = hash_pin(pin, &salt);
    let now_local = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO users (username, pin_hash, salt, full_name, role, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![username, pin_hash, salt, full_name, role, now_local],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "Username sudah digunakan.".to_string()
        } else {
            e.to_string()
        }
    })?;

    let id = conn.last_insert_rowid();

    Ok(UserProfile {
        id,
        username: username.to_string(),
        full_name: full_name.to_string(),
        role: role.to_string(),
        created_at: Some(now_local),
        last_login: None,
    })
}

/// Updates user PIN.
pub fn update_user_pin(
    conn: &Connection,
    user_id: i64,
    current_pin: Option<&str>,
    new_pin: &str,
    is_admin_override: bool,
) -> Result<(), String> {
    if new_pin.len() < 4 || new_pin.len() > 8 || !new_pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN baru harus berupa 4 hingga 8 digit angka.".to_string());
    }

    if !is_admin_override {
        let (pin_hash, salt) = conn
            .query_row(
                "SELECT pin_hash, salt FROM users WHERE id = ?1",
                params![user_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(|_| "Pengguna tidak ditemukan.".to_string())?;

        let cur = current_pin.ok_or_else(|| "PIN lama harus diisi.".to_string())?;
        if !verify_pin(cur, &salt, &pin_hash) {
            return Err("PIN lama Anda tidak sesuai.".to_string());
        }
    }

    let new_salt = generate_salt();
    let new_hash = hash_pin(new_pin, &new_salt);

    conn.execute(
        "UPDATE users SET pin_hash = ?1, salt = ?2 WHERE id = ?3",
        params![new_hash, new_salt, user_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Deletes a user (admin cannot delete own account or last admin).
pub fn delete_user(conn: &Connection, target_user_id: i64, current_user_id: i64) -> Result<(), String> {
    if target_user_id == current_user_id {
        return Err("Anda tidak dapat menghapus akun Anda sendiri saat sedang masuk.".to_string());
    }

    let role: String = conn
        .query_row(
            "SELECT role FROM users WHERE id = ?1",
            params![target_user_id],
            |row| row.get(0),
        )
        .map_err(|_| "Pengguna tidak ditemukan.".to_string())?;

    if role == "admin" {
        let admin_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE role = 'admin'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if admin_count <= 1 {
            return Err("Tidak dapat menghapus admin terakhir pada sistem.".to_string());
        }
    }

    conn.execute("DELETE FROM users WHERE id = ?1", params![target_user_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf2_compatibility() {
        let salt = "1b90e03b2784cb3ce769aed861872e45";
        let pin = "123456";
        let expected = "435fa6d6175c000396e8b7c618dbe9c87f9f32d5d6a977b34eba183f777400ef5b634c527a512e0b258fa75bf9e4073b318cb96eaef29c3685cd0b081c3de76d";
        assert_eq!(hash_pin(pin, salt), expected);
        assert!(verify_pin(pin, salt, expected));
        assert!(!verify_pin("wrong_pin", salt, expected));
    }
}
