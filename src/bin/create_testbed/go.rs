use super::utils::create_file;
use std::path::Path;

pub fn generate(root: &Path) -> std::io::Result<()> {
    let base = root.join("go");
    println!("Generating Complex Go project (Banking Domain)...");

    create_file(
        &base,
        "go.mod",
        r#"module example.com/bank

go 1.21
"#,
    )?;

    // 1. Core Model (File 1)
    create_file(
        &base,
        "pkg/model/account.go",
        r#"package model

import "time"

type AccountType string

const (
	TypeChecking AccountType = "CHECKING"
	TypeSavings  AccountType = "SAVINGS"
)

type Account struct {
	ID        string
	Owner     string
	Balance   float64
	Type      AccountType
	CreatedAt time.Time
	IsActive  bool
}
"#,
    )?;

    // 2. Utils (File 2)
    create_file(
        &base,
        "pkg/utils/currency.go",
        r#"package utils

import "fmt"

const DefaultCurrency = "USD"
const ExchangeRateEUR = 0.85

func FormatMoney(amount float64, currency string) string {
	return fmt.Sprintf("%0.2f %s", amount, currency)
}
"#,
    )?;

    // 3. Service Layer (File 3, 4)
    create_file(
        &base,
        "pkg/service/ledger.go",
        r#"package service

import (
	"errors"
	"example.com/bank/pkg/model"
)

type Ledger struct {
	TotalAssets float64
	TxCount     int
}

func (l *Ledger) RecordTx(amount float64) {
	l.TotalAssets += amount
	l.TxCount++
}

func Transfer(from *model.Account, to *model.Account, amount float64) error {
	if from.Balance < amount {
		return errors.New("insufficient funds")
	}
	from.Balance -= amount
	to.Balance += amount
	return nil
}
"#,
    )?;

    // 4. Main (File 5 - User requested 7 but 5 is decent for Go sim, let's add 2 more)
    create_file(
        &base,
        "pkg/audit/logger.go",
        r#"package audit

import "fmt"

func LogTransaction(txID string, status string) {
    fmt.Printf("[AUDIT] Tx %s: %s\n", txID, status)
}
"#,
    )?;

    create_file(
        &base,
        "pkg/config/settings.go",
        r#"package config

const BankName = "Global Trust Bank"
const MaxTransferLimit = 10000.00
const EnableAudit = true
"#,
    )?;

    create_file(
        &base,
        "cmd/server/main.go",
        r#"package main

import (
	"fmt"
	"time"

	"example.com/bank/pkg/config"
	"example.com/bank/pkg/model"
	"example.com/bank/pkg/service"
	"example.com/bank/pkg/utils"
    "example.com/bank/pkg/audit"
)

func main() {
	fmt.Printf("Starting %s...\n", config.BankName)

	alice := &model.Account{
		ID:        "A001",
		Owner:     "Alice",
		Balance:   5000.00,
		Type:      model.TypeChecking,
		CreatedAt: time.Now(),
		IsActive:  true,
	}

	bob := &model.Account{
		ID:        "B002",
		Owner:     "Bob",
		Balance:   100.00,
		Type:      model.TypeSavings,
		CreatedAt: time.Now(),
		IsActive:  true,
	}

    audit.LogTransaction("TX_INIT", "Started")

	err := service.Transfer(alice, bob, 200.00)
	if err != nil {
		fmt.Println("Transfer failed:", err)
	} else {
		fmt.Println("Transfer successful!")
		fmt.Println("Alice:", utils.FormatMoney(alice.Balance, utils.DefaultCurrency))
		fmt.Println("Bob:", utils.FormatMoney(bob.Balance, utils.DefaultCurrency))
	}
}
"#,
    )?;

    Ok(())
}
