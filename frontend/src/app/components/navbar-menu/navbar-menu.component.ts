import { Component, EventEmitter, OnInit, Output } from '@angular/core';
import { MatDialog } from '@angular/material/dialog';

import DocumentService from 'src/app/@core/services/document/document.service';
import { GlobalModel } from 'src/app/@core/models/global.model';
import { INativeService } from 'src/app/@core/services/native/native.interface';
import { SettingsComponent } from '../settings/settings.component';
import TEMPLATES from '../../../assets/docsTemplate.json';

const DOCS_TEMPLATE = TEMPLATES.docsTemplate;

@Component({
  selector: 'app-navbar-menu',
  templateUrl: './navbar-menu.component.html',
  styleUrls: ['./navbar-menu.component.scss'],
})
export class NavbarMenuComponent implements OnInit {
  @Output()
  readonly darkModeSwitched = new EventEmitter<boolean>();

  constructor(
    public globals: GlobalModel,
    private nativeService: INativeService,
    private documentService: DocumentService,
    private dialog: MatDialog,
  ) {}

  ngOnInit(): void {
    this.newDocument();
  }

  newDocument(): void {
    this.globals.loadTemplate(DOCS_TEMPLATE);
  }

  loadFile(): void {
    this.documentService.loadFile();
  }

  exportCB2File(): void {
    this.documentService.exportFile({ name: 'CaBr2', extensions: ['cb2'] }, DOCS_TEMPLATE);
  }

  exportPDFFile(): void {
    this.documentService.exportFile({ name: 'PDF', extensions: ['pdf'] }, DOCS_TEMPLATE);
  }

  openSettingsDialog(): void {
    const dialogRef = this.dialog.open(SettingsComponent);

    dialogRef.componentInstance.darkModeSwitched.subscribe((checked: boolean) => {
      this.darkModeSwitched.emit(checked);
    });
  }

  openManualDialog(): void {
    this.nativeService.openUrl('http://cabr2.de/anleitung.html');
  }
}
