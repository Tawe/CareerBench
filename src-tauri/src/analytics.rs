//! Analytics and insights module for job search metrics

use crate::db::get_connection;
use crate::errors::CareerBenchError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRates {
    pub application_to_interview: f64,
    pub interview_to_offer: f64,
    pub application_to_offer: f64,
    pub total_applications: i64,
    pub total_interviews: i64,
    pub total_offers: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeInStage {
    pub stage: String,
    pub average_days: f64,
    pub median_days: f64,
    pub min_days: Option<i64>,
    pub max_days: Option<i64>,
    pub sample_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelEffectiveness {
    pub channel: Option<String>,
    pub total_applications: i64,
    pub interviews: i64,
    pub offers: i64,
    pub interview_rate: f64,
    pub offer_rate: f64,
    pub average_time_to_interview: Option<f64>,
    pub average_time_to_offer: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Insight {
    pub category: String,
    pub title: String,
    pub message: String,
    pub priority: String, // "high", "medium", "low"
    pub actionable: bool,
}

/// Calculate conversion rates for applications
pub fn calculate_conversion_rates(
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<ConversionRates, CareerBenchError> {
    let conn = get_connection()?;

    let date_filter = if let (Some(start), Some(end)) = (start_date, end_date) {
        format!("AND date_saved >= '{}' AND date_saved <= '{}'", start, end)
    } else {
        String::new()
    };

    // Total applications
    let total_applications: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM applications WHERE archived = 0 {}", date_filter),
        [],
        |row| row.get(0),
    )?;

    // Total interviews (applications that reached Interviewing status)
    let total_interviews: i64 = conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM applications 
             WHERE archived = 0 AND status = 'Interviewing' {}",
            date_filter
        ),
        [],
        |row| row.get(0),
    )?;

    // Total offers
    let total_offers: i64 = conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM applications 
             WHERE archived = 0 AND status = 'Offer' {}",
            date_filter
        ),
        [],
        |row| row.get(0),
    )?;

    let application_to_interview = if total_applications > 0 {
        (total_interviews as f64 / total_applications as f64) * 100.0
    } else {
        0.0
    };

    let interview_to_offer = if total_interviews > 0 {
        (total_offers as f64 / total_interviews as f64) * 100.0
    } else {
        0.0
    };

    let application_to_offer = if total_applications > 0 {
        (total_offers as f64 / total_applications as f64) * 100.0
    } else {
        0.0
    };

    Ok(ConversionRates {
        application_to_interview,
        interview_to_offer,
        application_to_offer,
        total_applications,
        total_interviews,
        total_offers,
    })
}

/// Calculate average time spent in each stage
pub fn calculate_time_in_stage(
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<TimeInStage>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stages = Vec::new();
    let stage_list = vec!["Saved", "Applied", "Interviewing", "Offer", "Rejected", "Ghosted"];

    for stage in stage_list {
        let query = format!(
            r#"
            SELECT 
                julianday(COALESCE(
                    (SELECT MIN(event_date) FROM application_events 
                     WHERE application_id = a.id AND to_status = ?),
                    a.updated_at
                )) - julianday(a.date_saved) as days_in_stage
            FROM applications a
            WHERE a.status = ? AND a.archived = 0
            {}
            "#,
            if let (Some(start), Some(end)) = (start_date, end_date) {
                format!("AND a.date_saved >= '{}' AND a.date_saved <= '{}'", start, end)
            } else {
                String::new()
            }
        );

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([stage, stage], |row| {
            Ok(row.get::<_, Option<f64>>(0)?)
        })?;

        let mut days: Vec<f64> = Vec::new();
        for row_result in rows {
            if let Some(days_val) = row_result? {
                if days_val >= 0.0 {
                    days.push(days_val);
                }
            }
        }

        if !days.is_empty() {
            let sum: f64 = days.iter().sum();
            let count = days.len() as f64;
            let average = sum / count;

            days.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (days[mid - 1] + days[mid]) / 2.0
            } else {
                days[count as usize / 2]
            };

            let min_days = days.first().map(|d| d.round() as i64);
            let max_days = days.last().map(|d| d.round() as i64);

            stages.push(TimeInStage {
                stage: stage.to_string(),
                average_days: average.round(),
                median_days: median.round(),
                min_days,
                max_days,
                sample_size: days.len() as i64,
            });
        }
    }

    Ok(stages)
}

/// Analyze channel effectiveness
pub fn analyze_channel_effectiveness(
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<ChannelEffectiveness>, CareerBenchError> {
    let conn = get_connection()?;

    let date_filter = if let (Some(start), Some(end)) = (start_date, end_date) {
        format!("AND a.date_saved >= '{}' AND a.date_saved <= '{}'", start, end)
    } else {
        String::new()
    };

    let query = format!(
        r#"
        SELECT 
            a.channel,
            COUNT(DISTINCT a.id) as total_applications,
            SUM(CASE WHEN a.status = 'Interviewing' OR EXISTS (
                SELECT 1 FROM application_events e 
                WHERE e.application_id = a.id AND e.to_status = 'Interviewing'
            ) THEN 1 ELSE 0 END) as interviews,
            SUM(CASE WHEN a.status = 'Offer' THEN 1 ELSE 0 END) as offers,
            AVG(CASE WHEN EXISTS (
                SELECT 1 FROM application_events e 
                WHERE e.application_id = a.id AND e.to_status = 'Interviewing'
            ) THEN julianday((
                SELECT MIN(event_date) FROM application_events 
                WHERE application_id = a.id AND to_status = 'Interviewing'
            )) - julianday(a.date_saved) ELSE NULL END) as avg_time_to_interview,
            AVG(CASE WHEN a.status = 'Offer' THEN julianday((
                SELECT MIN(event_date) FROM application_events 
                WHERE application_id = a.id AND to_status = 'Offer'
            )) - julianday(a.date_saved) ELSE NULL END) as avg_time_to_offer
        FROM applications a
        WHERE a.archived = 0 {}
        GROUP BY a.channel
        ORDER BY total_applications DESC
        "#,
        date_filter
    );

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map([], |row| {
        let total: i64 = row.get(1)?;
        let interviews: i64 = row.get(2)?;
        let offers: i64 = row.get(3)?;
        let avg_time_to_interview: Option<f64> = row.get(4)?;
        let avg_time_to_offer: Option<f64> = row.get(5)?;

        let interview_rate = if total > 0 {
            (interviews as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let offer_rate = if total > 0 {
            (offers as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(ChannelEffectiveness {
            channel: row.get(0)?,
            total_applications: total,
            interviews,
            offers,
            interview_rate: interview_rate.round(),
            offer_rate: offer_rate.round(),
            average_time_to_interview: avg_time_to_interview.map(|d| d.round()),
            average_time_to_offer: avg_time_to_offer.map(|d| d.round()),
        })
    })?;

    let mut channels = Vec::new();
    for row_result in rows {
        channels.push(row_result?);
    }

    Ok(channels)
}

/// Generate AI insights based on patterns
pub fn generate_insights(
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<Insight>, CareerBenchError> {
    let mut insights = Vec::new();

    // Get conversion rates
    let conversion = calculate_conversion_rates(start_date, end_date)?;
    
    // Get time in stage
    let time_in_stage = calculate_time_in_stage(start_date, end_date)?;
    
    // Get channel effectiveness
    let channels = analyze_channel_effectiveness(start_date, end_date)?;

    // Insight 1: Low conversion rate
    if conversion.total_applications >= 10 {
        if conversion.application_to_interview < 10.0 {
            insights.push(Insight {
                category: "Conversion".to_string(),
                title: "Low Application-to-Interview Rate".to_string(),
                message: format!(
                    "Only {:.1}% of your applications are leading to interviews. Consider tailoring your resume and cover letters more specifically to each job posting.",
                    conversion.application_to_interview
                ),
                priority: "high".to_string(),
                actionable: true,
            });
        } else if conversion.application_to_interview < 20.0 {
            insights.push(Insight {
                category: "Conversion".to_string(),
                title: "Moderate Application-to-Interview Rate".to_string(),
                message: format!(
                    "Your interview rate is {:.1}%. There's room for improvement by focusing on quality over quantity and better targeting your applications.",
                    conversion.application_to_interview
                ),
                priority: "medium".to_string(),
                actionable: true,
            });
        }
    }

    // Insight 2: Interview to offer conversion
    if conversion.total_interviews >= 5 {
        if conversion.interview_to_offer < 20.0 {
            insights.push(Insight {
                category: "Conversion".to_string(),
                title: "Low Interview-to-Offer Rate".to_string(),
                message: format!(
                    "Only {:.1}% of your interviews are converting to offers. Consider practicing interview skills, researching companies more thoroughly, and preparing better questions.",
                    conversion.interview_to_offer
                ),
                priority: "high".to_string(),
                actionable: true,
            });
        }
    }

    // Insight 3: Time in stage insights
    for stage_data in &time_in_stage {
        if stage_data.stage == "Interviewing" && stage_data.average_days > 30.0 {
            insights.push(Insight {
                category: "Timing".to_string(),
                title: "Long Time in Interview Stage".to_string(),
                message: format!(
                    "Applications are spending an average of {:.0} days in the interviewing stage. Consider following up more proactively or diversifying your application strategy.",
                    stage_data.average_days
                ),
                priority: "medium".to_string(),
                actionable: true,
            });
        }

        if stage_data.stage == "Applied" && stage_data.average_days < 3.0 && stage_data.sample_size >= 5 {
            insights.push(Insight {
                category: "Timing".to_string(),
                title: "Quick Application Turnaround".to_string(),
                message: format!(
                    "You're applying quickly (avg {:.0} days), which is great! Make sure you're still tailoring each application.",
                    stage_data.average_days
                ),
                priority: "low".to_string(),
                actionable: false,
            });
        }
    }

    // Insight 4: Channel effectiveness
    if channels.len() > 1 {
        let best_channel = channels.iter()
            .max_by(|a, b| a.interview_rate.partial_cmp(&b.interview_rate).unwrap());
        
        let worst_channel = channels.iter()
            .filter(|c| c.total_applications >= 3)
            .min_by(|a, b| a.interview_rate.partial_cmp(&b.interview_rate).unwrap());

        if let (Some(best), Some(worst)) = (best_channel, worst_channel) {
            if best.interview_rate > worst.interview_rate + 10.0 {
                insights.push(Insight {
                    category: "Channel".to_string(),
                    title: "Channel Performance Difference".to_string(),
                    message: format!(
                        "Your best channel is '{}' ({:.1}% interview rate) vs '{}' ({:.1}%). Consider focusing more effort on your top-performing channels.",
                        best.channel.as_ref().unwrap_or(&"Unknown".to_string()),
                        best.interview_rate,
                        worst.channel.as_ref().unwrap_or(&"Unknown".to_string()),
                        worst.interview_rate
                    ),
                    priority: "medium".to_string(),
                    actionable: true,
                });
            }
        }
    }

    // Insight 5: Application volume
    if conversion.total_applications < 5 {
        insights.push(Insight {
            category: "Volume".to_string(),
            title: "Low Application Volume".to_string(),
            message: "You have fewer than 5 applications. Consider applying to more positions to increase your chances of success.".to_string(),
            priority: "medium".to_string(),
            actionable: true,
        });
    } else if conversion.total_applications >= 20 && conversion.application_to_interview < 15.0 {
        insights.push(Insight {
            category: "Volume".to_string(),
            title: "High Volume, Low Conversion".to_string(),
            message: "You're applying to many positions but getting few interviews. Focus on quality and better targeting rather than quantity.".to_string(),
            priority: "high".to_string(),
            actionable: true,
        });
    }

    // Insight 6: Offer success
    if conversion.total_offers > 0 && conversion.total_applications >= 10 {
        insights.push(Insight {
            category: "Success".to_string(),
            title: "Congratulations!".to_string(),
            message: format!(
                "You've received {} offer(s) from {} applications - a {:.1}% success rate!",
                conversion.total_offers,
                conversion.total_applications,
                conversion.application_to_offer
            ),
            priority: "low".to_string(),
            actionable: false,
        });
    }

    Ok(insights)
}
