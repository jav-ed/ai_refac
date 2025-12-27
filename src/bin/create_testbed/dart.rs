use std::fs;
use std::path::Path;
use crate::utils::create_file;

pub fn generate(root: &Path) -> std::io::Result<()> {
    let dart_dir = root.join("dart");
    println!("Generating Complex Dart project (App Store Domain)...");
    fs::create_dir_all(&dart_dir)?;

    // Core Project Structure
    let lib_models = dart_dir.join("lib/models");
    let lib_services = dart_dir.join("lib/services");
    let lib_ui_screens = dart_dir.join("lib/ui/screens");
    let lib_utils = dart_dir.join("lib/utils");

    fs::create_dir_all(&lib_models)?;
    fs::create_dir_all(&lib_services)?;
    fs::create_dir_all(&lib_ui_screens)?;
    fs::create_dir_all(&lib_utils)?;

    // 1. Models
    create_file(&lib_models, "app_model.dart", r#"
class AppModel {
  final String id;
  final String name;
  final String developer;
  final double rating;

  AppModel({
    required this.id,
    required this.name,
    required this.developer,
    required this.rating,
  });

  factory AppModel.fromJson(Map<String, dynamic> json) {
    return AppModel(
      id: json['id'],
      name: json['name'],
      developer: json['developer'],
      rating: json['rating'].toDouble(),
    );
  }
}
"#)?;

    // 2. Services
    create_file(&lib_services, "api_service.dart", r#"
import '../models/app_model.dart';

class ApiService {
  final String baseUrl = 'https://api.example.com';

  Future<List<AppModel>> fetchFeaturedApps() async {
    // Simulated API call
    return [
      AppModel(id: '1', name: 'Cool App', developer: 'DevOne', rating: 4.5),
      AppModel(id: '2', name: 'Super Game', developer: 'GameCo', rating: 4.8),
    ];
  }
}
"#)?;

    // 3. UI
    create_file(&lib_ui_screens, "home_screen.dart", r#"
import 'package:flutter/material.dart';
import '../../services/api_service.dart';
import '../../models/app_model.dart';

class HomeScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      app_bar: AppBar(title: Text('App Store')),
      body: Center(child: Text('Welcome to the Store')),
    );
  }
}
"#)?;

    // 4. Utils
    create_file(&lib_utils, "validators.dart", r#"
class Validators {
  static bool isValidEmail(String email) {
    return email.contains('@');
  }

  static bool isStrongPassword(String password) {
    return password.length >= 8;
  }
}
"#)?;

    // 5. Main entry & config
    create_file(&dart_dir.join("lib"), "main.dart", r#"
import 'package:flutter/material.dart';
import 'ui/screens/home_screen.dart';

void main() {
  runApp(MaterialApp(
    home: HomeScreen(),
  ));
}
"#)?;

    create_file(&dart_dir, "pubspec.yaml", r#"
name: app_store
description: A complex dart project testbed.
version: 1.0.0+1
environment:
  sdk: '>=3.0.0 <4.0.0'
dependencies:
  flutter:
    sdk: flutter
dev_dependencies:
  flutter_test:
    sdk: flutter
flutter:
  uses-material-design: true
"#)?;

    Ok(())
}
