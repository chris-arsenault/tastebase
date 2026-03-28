# migrate-dynamo.jq — transforms DynamoDB scan output into PostgreSQL INSERT statements

def esc: gsub("'"; "''");

def sql_str:
  if . == null then "NULL"
  else "'" + (. | esc) + "'"
  end;

def sql_str_nn:
  if . == null or . == "" then "''"
  else "'" + (. | esc) + "'"
  end;

def sql_str_or(default):
  if . == null or . == "" then (default | sql_str)
  else sql_str
  end;

def sql_int:
  if . == null then "NULL"
  else tostring
  end;

def sql_bool:
  if . == true then "TRUE"
  else "FALSE"
  end;

def sql_uuid:
  if . == null then "gen_random_uuid()"
  else "'" + . + "'::uuid"
  end;

def sql_date:
  if . == null then "CURRENT_DATE"
  else "'" + . + "'::date"
  end;

def sql_ts:
  if . == null then "now()"
  else "'" + . + "'::timestamptz"
  end;

def sql_enum(t):
  if . == null then "NULL"
  else "'" + . + "'::" + t
  end;

def sql_jsonb:
  if . == null then "NULL"
  else (. | tojson | esc) + "''::jsonb"
  end;

# Header
"
-- =============================================================
-- Tasting records
-- =============================================================",

(.Items[] |
  # Extract from DynamoDB attribute format
  (.id.S) as $id |
  (.createdAt.S) as $created_at |
  (.updatedAt.S) as $updated_at |
  (.status.S // "complete") as $status |
  (.productType.S) as $product_type |
  (.name.S // "") as $name |
  (.maker.S // "") as $maker |
  (.date.S) as $date |
  (.score.N // null) as $score |
  (.style.S // "") as $style |
  (.heatUser.N // null) as $heat_user |
  (.heatVendor.N // null) as $heat_vendor |
  (.refreshing.N // null) as $refreshing |
  (.sweet.N // null) as $sweet |
  (.tastingNotesUser.S // "") as $notes_user |
  (.tastingNotesVendor.S // "") as $notes_vendor |
  (.productUrl.S // "") as $product_url |
  (.imageUrl.S) as $image_url |
  (.imageKey.S) as $image_key |
  (.ingredientsImageUrl.S) as $ing_image_url |
  (.ingredientsImageKey.S) as $ing_image_key |
  (.nutritionImageUrl.S) as $nut_image_url |
  (.nutritionImageKey.S) as $nut_image_key |
  (.nutritionFacts.M // null) as $nut_facts_raw |
  (if .ingredients.L then [.ingredients.L[].S] else null end) as $ingredients |
  (.voiceKey.S) as $voice_key |
  (.voiceTranscript.S) as $voice_transcript |
  (.processingError.S) as $proc_error |
  (.needsAttention.BOOL // false) as $needs_attn |
  (.attentionReason.S) as $attn_reason |
  (.createdBy.S // "unknown") as $created_by |

  # Build nutrition facts as a clean JSON object (strip DynamoDB .S/.N wrappers)
  (if $nut_facts_raw then
    ({} |
      if $nut_facts_raw.servingSize.S then . + {servingSize: $nut_facts_raw.servingSize.S} else . end |
      if $nut_facts_raw.calories.N then . + {calories: ($nut_facts_raw.calories.N | tonumber)} else . end |
      if $nut_facts_raw.totalFat.S then . + {totalFat: $nut_facts_raw.totalFat.S} else . end |
      if $nut_facts_raw.sodium.S then . + {sodium: $nut_facts_raw.sodium.S} else . end |
      if $nut_facts_raw.totalCarbs.S then . + {totalCarbs: $nut_facts_raw.totalCarbs.S} else . end |
      if $nut_facts_raw.sugars.S then . + {sugars: $nut_facts_raw.sugars.S} else . end |
      if $nut_facts_raw.protein.S then . + {protein: $nut_facts_raw.protein.S} else . end
    )
  else null end) as $nutrition_facts |

  # Build ingredients array literal
  (if $ingredients then
    "ARRAY[" + ([$ingredients[] | "'" + (. | gsub("'"; "''")) + "'"] | join(",")) + "]::text[]"
  else "NULL" end) as $ing_sql |

  # Build nutrition_facts jsonb literal
  (if $nutrition_facts then
    "'" + ($nutrition_facts | tojson | gsub("'"; "''")) + "'::jsonb"
  else "NULL" end) as $nf_sql |

  "
INSERT INTO tastings (
  id, user_id, product_type, name, maker, date, score, style,
  heat_user, heat_vendor, refreshing, sweet,
  tasting_notes_user, tasting_notes_vendor, product_url,
  image_url, image_key,
  ingredients_image_url, ingredients_image_key,
  nutrition_image_url, nutrition_image_key,
  nutrition_facts, ingredients,
  voice_key, voice_transcript,
  status, processing_error, needs_attention, attention_reason,
  created_at, updated_at
)
SELECT
  \($id | sql_uuid),
  cu.user_id,
  \($product_type | sql_enum("product_type")),
  \($name | sql_str_nn),
  \($maker | sql_str_nn),
  \($date | sql_date),
  \($score | sql_int),
  \($style | sql_str_nn),
  \($heat_user | sql_int),
  \($heat_vendor | sql_int),
  \($refreshing | sql_int),
  \($sweet | sql_int),
  \($notes_user | sql_str_nn),
  \($notes_vendor | sql_str_nn),
  \($product_url | sql_str_nn),
  \($image_url | sql_str),
  \($image_key | sql_str),
  \($ing_image_url | sql_str),
  \($ing_image_key | sql_str),
  \($nut_image_url | sql_str),
  \($nut_image_key | sql_str),
  \($nf_sql),
  \($ing_sql),
  \($voice_key | sql_str),
  \($voice_transcript | sql_str),
  \($status | sql_enum("processing_status")),
  \($proc_error | sql_str),
  \($needs_attn | sql_bool),
  \($attn_reason | sql_str),
  \($created_at | sql_ts),
  \($updated_at | sql_ts)
FROM cognito_users cu
WHERE cu.cognito_sub = \($created_by | sql_str)
ON CONFLICT (id) DO NOTHING;"
)
