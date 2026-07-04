SELECT
    o.id,
    o.amount,
    c.name
FROM [analytics].[marts_finance].[fct_orders] AS o
JOIN marts.dim_customers AS c ON c.id = o.customer_id
LEFT JOIN dbo.legacy_unmapped_table u ON u.id = o.id
WHERE o.amount > 100;

SELECT * FROM dim_customers;
