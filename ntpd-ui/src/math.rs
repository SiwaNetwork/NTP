//! Математика для предсказания ухода часов (упростил)
//! 
//! Считаем линейную регрессию - находим тренд изменения времени.

/// Результат расчета регрессии
pub struct RegressionResult {
    /// Наклон - насколько быстро меняется время (сек/сек)
    /// Положительное = часы спешат, отрицательное = отстают
    pub slope: f64,
    
    /// Начальное значение
    pub intercept: f64,
    
    /// Качество модели (0.0 - 1.0, чем ближе к 1.0, тем лучше)
    pub r_squared: f64,
}

/// Считаем линейную регрессию методом наименьших квадратов
/// 
/// Находим прямую линию, которая лучше всего описывает данные.
/// Возвращаем None, если точек меньше 2.
pub fn calculate_linear_regression(y: &[f64]) -> Option<RegressionResult> {
    let n = y.len() as f64;
    
    // Нужно минимум 2 точки
    if n < 2.0 {
        return None;
    }

    // Собираем суммы для формул
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;

    // Первый проход - считаем все суммы
    for (i, &val) in y.iter().enumerate() {
        let x = i as f64;
        sum_x += x;
        sum_y += val;
        sum_xy += x * val;
        sum_xx += x * x;
    }

    // Формула метода наименьших квадратов
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    // Считаем R² - насколько хорошо модель описывает данные
    let avg_y = sum_y / n;
    let mut ss_tot = 0.0;  // общая дисперсия
    let mut ss_res = 0.0;  // необъясненная дисперсия

    // Второй проход - считаем дисперсии
    for (i, &val) in y.iter().enumerate() {
        let x = i as f64;
        let prediction = slope * x + intercept;
        
        ss_res += (val - prediction).powi(2);
        ss_tot += (val - avg_y).powi(2);
    }

    // R² = 1 - (необъясненная / общая)
    let r_squared = if ss_tot == 0.0 {
        1.0  // если все значения одинаковы, модель идеальна
    } else {
        1.0 - (ss_res / ss_tot)
    };

    Some(RegressionResult {
        slope,
        intercept,
        r_squared,
    })
}

/// Вычисляет jitter (стандартное отклонение) из истории offset'ов
/// 
/// Jitter показывает вариативность изменения времени.
/// Чем меньше jitter, тем стабильнее синхронизация.
pub fn calculate_jitter(data: &[f64]) -> Option<f64> {
    let n = data.len() as f64;
    
    // Нужно минимум 2 точки для расчета
    if n < 2.0 {
        return None;
    }
    
    // Считаем среднее значение
    let mean = data.iter().sum::<f64>() / n;
    
    // Считаем дисперсию (средний квадрат отклонений)
    let variance = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / n;
    
    // Jitter = стандартное отклонение = корень из дисперсии
    Some(variance.sqrt())
}
