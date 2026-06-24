//! Content domain integration tests — packages, gallery groups, gallery.

mod common;

use common::setup;
use paidang_rs_server::domain::packages::dto::{CreatePackageReq, ListQuery, UpdatePackageReq};
use paidang_rs_server::domain::packages::service;

// ── Packages ──────────────────────────────────────────────

#[tokio::test]
async fn create_and_read_package() {
    let ctx = setup().await;

    let body = CreatePackageReq {
        name: "婚纱照".into(),
        subtitle: None,
        category: Some("wedding".into()),
        price: 299900,
        original_price: Some(399900),
        deposit: Some(50000),
        cover_image: None,
        description: None,
        service_items: None,
        suitable_people: None,
        shooting_location: None,
        validity_days: None,
        sort_order: None,
        is_hot: None,
        is_recommend: None,
        status: Some(1),
    };
    let created = service::create_package(&ctx.state, &body, 1).await.unwrap();
    assert!(created.package_id > 0);
    assert_eq!(created.name, "婚纱照");

    let found = service::read_package(&ctx.state, created.package_id)
        .await
        .unwrap();
    assert_eq!(found.price, 299900);
}

#[tokio::test]
async fn list_packages_pagination() {
    let ctx = setup().await;

    // Create 5 packages
    for i in 0..5 {
        let body = CreatePackageReq {
            name: format!("Package {i}"),
            subtitle: None,
            category: None,
            price: (i + 1) * 10000,
            original_price: None,
            deposit: None,
            cover_image: None,
            description: None,
            service_items: None,
            suitable_people: None,
            shooting_location: None,
            validity_days: None,
            sort_order: Some(i),
            is_hot: None,
            is_recommend: None,
            status: Some(1),
        };
        service::create_package(&ctx.state, &body, 1).await.unwrap();
    }

    let query = ListQuery {
        page: Some(1),
        page_size: Some(10),
        category: None,
        status: Some(1),
    };
    let (rows, total) = service::list_packages(&ctx.state, &query).await.unwrap();
    assert_eq!(total, 5);
    assert_eq!(rows.len(), 5);
}

#[tokio::test]
async fn update_package() {
    let ctx = setup().await;

    let body = CreatePackageReq {
        name: "原始名称".into(),
        subtitle: None,
        category: None,
        price: 10000,
        original_price: None,
        deposit: None,
        cover_image: None,
        description: None,
        service_items: None,
        suitable_people: None,
        shooting_location: None,
        validity_days: None,
        sort_order: None,
        is_hot: None,
        is_recommend: None,
        status: Some(1),
    };
    let created = service::create_package(&ctx.state, &body, 1).await.unwrap();

    let update = UpdatePackageReq {
        name: Some("更新名称".into()),
        price: Some(20000),
        ..Default::default()
    };
    let updated = service::update_package(&ctx.state, created.package_id, &update, 1)
        .await
        .unwrap();
    assert_eq!(updated.name, "更新名称");
    assert_eq!(updated.price, 20000);
}

#[tokio::test]
async fn delete_package() {
    let ctx = setup().await;

    let body = CreatePackageReq {
        name: "待删除".into(),
        subtitle: None,
        category: None,
        price: 100,
        original_price: None,
        deposit: None,
        cover_image: None,
        description: None,
        service_items: None,
        suitable_people: None,
        shooting_location: None,
        validity_days: None,
        sort_order: None,
        is_hot: None,
        is_recommend: None,
        status: Some(1),
    };
    let created = service::create_package(&ctx.state, &body, 1).await.unwrap();

    service::delete_package(&ctx.state, created.package_id)
        .await
        .unwrap();

    let result = service::read_package(&ctx.state, created.package_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("不存在"));
}

// ── Package Items ─────────────────────────────────────────

#[tokio::test]
async fn create_and_list_items() {
    let ctx = setup().await;

    // Create a package first
    let body = CreatePackageReq {
        name: "含项目的套餐".into(),
        subtitle: None,
        category: None,
        price: 10000,
        original_price: None,
        deposit: None,
        cover_image: None,
        description: None,
        service_items: None,
        suitable_people: None,
        shooting_location: None,
        validity_days: None,
        sort_order: None,
        is_hot: None,
        is_recommend: None,
        status: Some(1),
    };
    let pkg = service::create_package(&ctx.state, &body, 1).await.unwrap();

    // Create items
    use paidang_rs_server::domain::packages::dto::CreateItemReq;
    let item = CreateItemReq {
        package_id: pkg.package_id,
        item_type: "photo".into(),
        item_name: "精修照片".into(),
        quantity: Some(30),
        unit: Some("张".into()),
        item_value: None,
        sort_order: Some(1),
        is_default: Some(1),
    };
    let created = service::create_item(&ctx.state, &item).await.unwrap();
    assert_eq!(created.item_name, "精修照片");

    let items = service::list_items(&ctx.state, pkg.package_id).await.unwrap();
    assert_eq!(items.len(), 1);
}
