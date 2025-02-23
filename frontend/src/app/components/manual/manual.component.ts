import { Component, Inject, OnInit, ViewEncapsulation } from '@angular/core';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';

@Component({
  selector: 'app-manual',
  templateUrl: './manual.component.html',
  styleUrls: ['./manual.component.scss'],
  encapsulation: ViewEncapsulation.None,
})
export class ManualComponent implements OnInit {
  constructor(
    public dialogRef: MatDialogRef<ManualComponent>,
    @Inject(MAT_DIALOG_DATA) public data: { content: string },
  ) {}

  ngOnInit(): void {}
}
