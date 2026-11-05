/*
 * SPDX-FileCopyrightText: 2026 Acme Corp.
 * SPDX-License-Identifier: Apache-2.0
 */

export const retry = async <T>(fn: () => Promise<T>): Promise<T> => fn();
