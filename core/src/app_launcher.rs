// app_launcher.rs

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Находит исполняемый файл в PATH
/// 
/// Если указано просто имя (например, "notepad"), ищет в PATH.
/// Если указан полный путь, проверяет существование файла.
pub fn find_executable(app: &str) -> Result<PathBuf> {
    let path = Path::new(app);
    
    // Если указан полный или относительный путь
    if path.is_absolute() || app.contains('\\') || app.contains('/') {
        let path_buf = PathBuf::from(app);
        
        // Проверяем существование
        if !path_buf.exists() {
            return Err(anyhow!(
                "Файл приложения не найден: {}\n\n\
                 💡 Убедитесь, что:\n\
                 - Путь указан правильно\n\
                 - Файл существует\n\
                 - У вас есть права доступа",
                app
            ));
        }
        
        return Ok(path_buf);
    }
    
    // Ищем в PATH
    find_in_path(app).ok_or_else(|| anyhow!(
        "Приложение '{}' не найдено в PATH\n\n\
         💡 Варианты:\n\
         - Укажите полный путь: \"C:\\Windows\\System32\\notepad.exe\"\n\
         - Добавьте приложение в PATH\n\
         - Используйте известное имя: \"notepad\", \"calc\", \"mspaint\"",
        app
    ))
}

/// Ищет исполняемый файл в переменной окружения PATH
fn find_in_path(app_name: &str) -> Option<PathBuf> {
    // Добавляем .exe если нет расширения
    let app_with_ext = if app_name.ends_with(".exe") {
        app_name.to_string()
    } else {
        format!("{}.exe", app_name)
    };
    
    // Получаем PATH
    let path_var = std::env::var("PATH").ok()?;
    
    // Ищем в каждой директории, пропуская пустые записи (trailing `;` или `;;`)
    for dir in path_var.split(';').filter(|d| !d.is_empty()) {
        let candidate = Path::new(dir).join(&app_with_ext);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    
    None
}

/// Запускает приложение
/// 
/// Возвращает PID запущенного процесса
pub fn launch_app(app_path: &Path, args: &[&str]) -> Result<u32> {
    println!("🚀 Запуск приложения: {}", app_path.display());
    
    let mut cmd = Command::new(app_path);
    cmd.args(args);
    
    let child = cmd.spawn().map_err(|e| anyhow!(
        "Не удалось запустить приложение '{}': {}\n\n\
         💡 Проверьте:\n\
         - Файл является исполняемым\n\
         - У вас есть права на запуск",
        app_path.display(),
        e
    ))?;
    
    let pid = child.id();
    println!("✅ Приложение запущено (PID: {})", pid);
    
    Ok(pid)
}

/// Запускает приложение и ждёт готовности окна
pub fn launch_app_and_wait(app_path: &Path, args: &[&str], wait_ms: u64) -> Result<u32> {
    let pid = launch_app(app_path, args)?;
    
    println!("⏳ Ожидание загрузки приложения ({}ms)...", wait_ms);
    std::thread::sleep(std::time::Duration::from_millis(wait_ms));
    
    Ok(pid)
}

/// Парсит аргумент приложения из командной строки
/// 
/// Поддерживаемые форматы:
/// - "notepad" → ищем в PATH
/// - "C:\path\to\app.exe" → полный путь
/// - "notepad file.txt" → приложение с аргументами
pub fn parse_app_arg(app_arg: &str) -> Result<(PathBuf, Vec<String>)> {
    // Разделяем по пробелам: "notepad file.txt" → ["notepad", "file.txt"]
    let parts: Vec<&str> = app_arg.split_whitespace().collect();
    
    if parts.is_empty() {
        return Err(anyhow!("Пустой аргумент приложения"));
    }
    
    let app_name = parts[0];
    let app_args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    
    // Ищем приложение
    let app_path = find_executable(app_name)?;
    
    Ok((app_path, app_args))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_find_notepad_in_path() {
        // Notepad должен быть в PATH на Windows
        let result = find_executable("notepad");
        assert!(result.is_ok(), "Notepad должен быть в PATH на Windows");
        
        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("notepad"));
    }
    
    #[test]
    fn test_find_calc() {
        let result = find_executable("calc");
        assert!(result.is_ok(), "Calc должен быть в PATH на Windows");
    }
    
    #[test]
    fn test_full_path_exists() {
        let result = find_executable("C:\\Windows\\System32\\notepad.exe");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_full_path_not_exists() {
        let result = find_executable("C:\\NonExistent\\app.exe");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_app_not_in_path() {
        let result = find_executable("nonexistent_app_xyz123");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("не найдено в PATH"));
    }
    
    #[test]
    fn test_parse_app_arg_simple() {
        let (path, args) = parse_app_arg("notepad").unwrap();
        assert!(path.to_string_lossy().contains("notepad"));
        assert!(args.is_empty());
    }
    
    #[test]
    fn test_parse_app_arg_with_args() {
        let (path, args) = parse_app_arg("notepad file.txt").unwrap();
        assert!(path.to_string_lossy().contains("notepad"));
        assert_eq!(args, vec!["file.txt"]);
    }
}
